# Task: XR camera-offset compensation for pose-driven content

Date: 2026-09-08

Status: active design and bug task.

This task records the camera-offset contract discovered while using a
non-humanoid car model under `InputXR` and `AvatarControl` in
[`examples/mittens-corp.mms`](../../examples/mittens-corp.mms).

Related context:

- [`docs/analysis/cameraxr-avatar-alignment.md`](../analysis/cameraxr-avatar-alignment.md)
- [`docs/task/avatar-control-head-driven-redesign.md`](./avatar-control-head-driven-redesign.md)
- [`docs/task/avatar-control-simple-humanoid-body-follow.md`](./avatar-control-simple-humanoid-body-follow.md)
- [`docs/task/avatar-control-desktop-vr-facing-and-eye-offset-unification.md`](./avatar-control-desktop-vr-facing-and-eye-offset-unification.md)
- [`src/engine/ecs/system/openxr_system.rs`](../../src/engine/ecs/system/openxr_system.rs)
- [`src/engine/ecs/system/avatar_control_system.rs`](../../src/engine/ecs/system/avatar_control_system.rs)
- [`src/engine/ecs/system/humanoid_bone_map_system.rs`](../../src/engine/ecs/system/humanoid_bone_map_system.rs)

---

## 1. Observed issue

The `mittens-corp` XR rig places a car and a wrapped `CameraXR` beneath the
same pose-driven `AvatarControl` subtree:

```text
InputXR
  -> tracked transform
       -> AvatarControl
            -> car model
            -> T.position(camera_offset)
                 -> CameraXR
```

Changing the camera wrapper to an obviously large offset, for example
`T.position(0, 17, 0) { CXR {} }`, does not change the rendered XR camera's
position relative to the car. Moving the wrapped camera outside
`AvatarControl`, while leaving it beneath the same `InputXR`, also does not
change the result.

The car's measured render bounds correctly contain its visible geometry. The
failure is therefore not caused by bad bounds or an incorrect model origin.

---

## 2. Current engine behavior

Two existing policies combine to produce the bug.

### 2.1 XR rendering deliberately keeps the physical eye pose authoritative

`OpenXRSystem::xr_rig_origin_world` finds the active camera's `InputXR`
ancestor and uses the world transform above the tracked transform as the XR
rig origin. It does not use the wrapped `CameraXR` node's world transform.
The final rendered eye transform is then:

```text
rendered_eye_world = xr_rig_origin_world * raw_openxr_eye_pose
```

This avoids applying the tracked HMD pose twice. It also means that an
authored transform around `CameraXR` cannot directly move the rendered XR
view while it remains under that `InputXR` path.

### 2.2 Camera-offset consumption currently depends on humanoid initialization

`AvatarControlSystem` discovers a wrapped camera offset while initializing a
humanoid. It uses that offset when placing the visible head and body relative
to the fixed physical camera.

Initialization currently requires:

- a skinned GLTF beneath `AvatarControl`, and
- a resolved humanoid head bone.

The car is an ordinary, unskinned GLTF with no humanoid head. The humanoid map
request therefore returns no usable model, initialization exits early, and no
system consumes the authored camera offset. The model continues to inherit the
tracked transform while the rendered camera continues to use the raw OpenXR
eye pose.

Bounds and capsule inference happen later and cannot repair this relationship.

---

## 3. Decided semantic contract

A transform wrapping `CameraXR` beneath a pose driver represents the authored
camera offset relative to the content driven by that pose.

Let:

- `P` be the tracked pose in world space,
- `C` be the authored camera-offset transform, and
- `A` be a model's own authored local transform.

The intended relationship is:

```text
rendered XR camera = P
pose-driven model content = P * inverse(C) * A
```

In the initial implementation, translation is the required behavior:

```text
model_translation_compensation = -camera_offset_translation
```

This contract applies whether the model is:

- an unskinned rigid model such as a vehicle,
- a skinned non-humanoid model, or
- a humanoid with a mapped head bone.

The physical headset remains authoritative. The camera wrapper does not
artificially move the user's tracked eyes; instead, its inverse moves the
pose-driven visual content so the camera occupies the authored location
relative to that content.

---

## 4. Scope of compensation

"Everything else driven by the same pose" means the body/model content whose
placement is derived from the head pose. It does not mean every tracked input
sample in the subtree.

Apply inverse camera-offset compensation to:

- the rigid model root,
- the humanoid body/model root, and
- other visual body content explicitly owned by the same pose-driven rig.

Do not apply it to independently tracked endpoints:

- OpenXR eye poses,
- XR hand/controller target transforms,
- eye-tracking samples, or
- other raw tracking-space inputs.

Those targets remain at their physical tracked positions. A compensated
avatar skeleton may solve toward them.

---

## 5. Humanoid specialization

A mapped humanoid head adds calibration information; it does not enable the
base camera-offset meaning.

For a rigid or non-humanoid model:

```text
model_world = P * inverse(C) * A
```

For a humanoid, let `H` represent the mapped head/camera anchor in model space.
The body additionally needs the inverse head-anchor calibration:

```text
model_world = P * inverse(C) * inverse(H) * A
head_target_world = P * inverse(C)
```

Existing head displacement, body yaw follow, neck policy, and arm IK may then
operate from those calibrated targets. Missing head mapping must skip only
the humanoid-specific work; it must not discard generic camera-offset
compensation.

---

## 6. Target runtime structure

Prefer inserting a runtime-owned compensation transform instead of rewriting
the model root's authored transform:

```text
tracked pose P
  |-- camera wrapper C
  |     `-- CameraXR
  |-- generated model compensation inverse(C)
  |     `-- authored model root A
  `-- independently tracked controller targets
```

This preserves authoring such as the car's small ground/model-origin
adjustment. It also gives the inverse offset one explicit owner and avoids
baking runtime calibration into serialized scene values.

For humanoids, the generated body pipeline and head target must consume the
same captured camera offset exactly once. The refactor must not apply both the
new generic compensation and the old eye-offset adjustment to the same model
branch.

The authored offset must be captured before `AvatarControlSystem` reparents a
camera path beneath a mapped head/camera anchor.

---

## 7. Open design decisions

The following are intentionally not decided yet:

1. Whether the first implementation supports translation only or the complete
   inverse TRS. Translation is required. Full rotation/scale semantics should
   be added only with explicit tests and a clear scale policy.
2. Whether generic compensation belongs in `AvatarControlSystem`, in a shared
   pose-driver utility/system, or in a new component that can serve non-AVC
   pose-driven rigs.
3. How the compensated content root is selected when an `InputXR` subtree has
   multiple models or multiple enabled `CameraXR` components.
4. Whether live edits to the camera wrapper update compensation immediately or
   whether the offset is captured only during rig initialization.
5. How collision roots follow the generated compensation node without
   changing the existing locomotion movement target.
6. Whether a non-humanoid `AvatarControl` should remain a supported public
   topology or whether the generic behavior should ultimately be named and
   exposed independently of humanoid avatar control.

---

## 8. Proposed implementation phases

### Phase 1: Extract and retain the camera offset independently of humanoid mapping

- Discover the active wrapped `CameraXR` path before requesting a humanoid map.
- Retain the authored local translation and camera-path component IDs.
- Distinguish "model still loading" from "loaded model has no skin/head map"
  so rigid initialization does not race GLTF loading.

### Phase 2: Add rigid/non-humanoid compensation

- Create a runtime-only inverse-offset transform above the pose-driven model
  content.
- Preserve the model root's authored local transform.
- Do not create head, neck, spine, or arm logic when no mapped head exists.
- Keep XR controller transforms in physical tracking space.

### Phase 3: Refactor humanoid initialization onto the shared contract

- Reuse the same retained camera offset for head-target and body calibration.
- Ensure the offset is applied exactly once per visual branch.
- Preserve existing humanoid camera/head alignment and body-follow behavior.

### Phase 4: Define live-update and full-TRS behavior

- Decide whether runtime camera-wrapper edits are supported.
- If rotation is supported, use a mathematically complete inverse transform;
  do not negate Euler angles or translation independently.
- Define or reject non-unit camera-wrapper scale explicitly.

---

## 9. Required regression coverage

Add focused tests for these cases:

1. **Rigid GLTF under AVC:** changing camera-wrapper Y from `3` to `17`
   produces the corresponding inverse model displacement.
2. **Authored model offset preservation:** camera compensation does not erase a
   model root offset such as `[0, 0.1, 0]`.
3. **No camera offset:** an identity camera wrapper leaves model placement
   unchanged.
4. **Humanoid mapped head:** the existing head/body calibration remains stable
   and the camera offset is not applied twice.
5. **Unmapped or missing head:** generic compensation still initializes and no
   humanoid IK/splice nodes are created.
6. **Tracked controllers:** controller world poses remain unchanged when the
   camera offset changes.
7. **Nested camera outside AVC:** the documented pose-driver ownership rule is
   consistent when the camera remains beneath the same `InputXR` but is not a
   direct AVC child.
8. **Runtime topology:** generated compensation nodes are non-serialized and
   are removed with their owning rig.

The `mittens-corp` example remains the visual proof scene. A large temporary
camera Y offset should make the inverse model movement unmistakable in XR.

---

## 10. Acceptance criteria

This task is complete when:

- a wrapped `CameraXR` translation has the same semantic meaning for rigid,
  skinned non-humanoid, and humanoid models,
- non-humanoid behavior no longer depends on fabricating a head bone or
  humanoid map,
- the rendered XR camera remains based on the physical OpenXR eye pose,
- pose-driven body/model content receives the inverse authored camera offset,
- independently tracked hands/controllers do not receive that compensation,
- authored model-root transforms are preserved,
- humanoid head-anchor calibration remains an additional specialization,
- the offset is applied exactly once, and
- automated regression tests cover both the car and humanoid paths.
