# Task: desktop occupancy mount pose and input-authority first slice

Status: planned, 2026-09-12.

## Goal

Deliver one coherent desktop Rider-to-Mountable path in
`examples/mittens-corp-desktop.mms`:

- entering the car places the Rider at the destination mount point with the
  facing authored by that point's Transform;
- pedestrian translation stops while mounted;
- ordinary desktop mouse and arrow look remains active;
- vehicle movement controls remain a separate authored control layer;
- dismount restores exactly the pedestrian translation state that existed
  before mounting.

This slice establishes two boundaries before the attachment model becomes more
general:

1. Both selected mount points are oriented coordinate frames.  Their relative
   authored rotations, not a second `Same`/`Opposed` flag, determine mounted
   facing.
2. A desktop `Input` pose driver has independently controllable translation and
   rotation capabilities beneath its master lifecycle gate.

## Decided mount-frame convention

Do not introduce `MountFacing::Same` or `MountFacing::Opposed`.

The destination mount point's Transform is authored in the desired mounted
frame, and the Rider-side mount point encodes the controlled object's local
attachment frame.  Alignment must solve the Rider root pose that makes those
two selected frames coincide on the enabled channels.  This keeps position,
arbitrary angular offsets, and facing in inspectable authored frames and avoids
contradictory sources of orientation truth.

For the current horizontal vehicle behavior:

- the destination mount point supplies target position and target world yaw;
- the Rider-side mount point's yaw relative to the movement root is removed
  when solving the required root yaw;
- the Rider movement root remains free of pitch and roll;
- the Rider-side mount point supplies the positional offset used to place the
  root;
- desktop local look remains below the movement root and continues after the
  base yaw is established.

In yaw-only notation, if `A` is the Rider-side mount frame relative to the
movement root and `D` is the destination mount frame in world space, solve:

```text
root_yaw = world_yaw(D) - relative_yaw(A)
```

Then rotate the Rider-side positional offset by that solved root yaw when
placing the root.  This is the horizontal projection of ordinary frame
alignment, not a special forward-axis flip.

In the current fixture, `bisket_first_person_camera_slot` contains a π yaw that
bridges Bisket's head-local `+Z` facing to `Camera3D`'s local `-Z` view-forward.
`left_display_car_desktop_mount` supplies the desired mounted frame.  Generic
alignment must honor both rotations; do not rotate the destination point merely
to cancel a source-frame offset that the solver ignored.

This is not an OpenXR-versus-window projection difference.  OpenXR view poses
and the engine's window `Camera3D` both use local `-Z` view-forward.  The
half-turn exists at the imported avatar/head-to-camera boundary, and it becomes
visible during mounting because the current horizontal alignment uses the
destination yaw without accounting for the Rider-side frame's relative yaw.

The fact that the corresponding XR example already faces correctly does not
validate the current frame-alignment calculation.  The two examples use
different Rider-side points: the desktop fixture selects the π-rotated
`bisket_first_person_camera_slot`, while XR selects
`bisket_rider_cxr_anchor`, which has no authored yaw.  More importantly, the
active XR camera path under `InputXR` renders each eye as the locomotion rig
origin composed with the live OpenXR eye pose; it does not use the CXR node's
inherited avatar/head rotation as the rendered camera basis.  Both car-side
targets currently have the same local position and no local rotation, despite
their distinct names.  Consequently, setting the locomotion root to the
destination yaw is sufficient for the XR rendered view, while the desktop
camera still inherits its additional π slot rotation and exposes the error.

The current XR Rider anchor sits in the avatar/head hierarchy while rendered
head orientation comes from the separately composed live OpenXR pose.  Neither
that hierarchy-dependent orientation nor the live tracked pose should be
canceled into the locomotion root.  This desktop slice must preserve the
existing XR rule while adding source-frame yaw alignment for the desktop
virtual-look driver.  A later first-class mount-point model should give XR a
stable authored Rider-side mount frame separate from its live head-pose probe,
removing the need to infer those meanings from camera plumbing.

When both endpoints become first-class `MountPoint` components, the same rule
extends to complete frame alignment: align the selected source frame to the
selected destination frame according to the chosen translation/rotation
channels.  Any asset-axis or mating-axis correction is authored into one
endpoint frame rather than represented by a parallel facing enum.

## Initial alignment versus ongoing following

Mount-point orientation answers the desired alignment frame.  It does not, by
itself, answer which transform channels remain coupled after attachment.  Keep
those concepts separate.

This first slice preserves and makes testable the existing ongoing behavior:

- mount commit establishes the horizontal Rider root pose;
- the active mount retains the relative pose established at commit;
- subsequent destination movement carries the Rider through the existing
  `TransformParent` relationship;
- local desktop look remains an additional descendant rotation rather than
  rewriting the mount point or vehicle;
- dismount removes the follow relationship and restores an independent Rider
  world pose.

Do not add configurable yaw-only/full/position-only follow modes in this slice.
The active attachment edge is the future owner of that constraint policy, but
we need a second concrete behavior before exposing an authored mode matrix.

## Desktop Input capability contract

`InputComponent` is the canonical owner of these gates:

```text
enabled                 master pose-driver lifecycle gate
translation_enabled     built-in WASD/R/F translation
rotation_enabled        built-in mouse/arrow/Q/E rotation
```

All default to `true`.  Effective behavior is:

```text
translation active = enabled && translation_enabled
rotation active    = enabled && rotation_enabled
```

Expose component/live operations with the ordinary naming conventions,
including:

```text
set_translation_enabled(bool)
set_rotation_enabled(bool)
```

and corresponding reads/builders.  Existing `enable()` / `disable()` remains
the master operation and must not be used for translation-only handoff.

`InputTransformMode` should retain only transform interpretation choices such
as forward axis, roll axis, translation basis, and FPS reconstruction.
`rotation_enabled` must not remain as a second independent source of truth
there.  Migrate `InputTransformMode.rotation_disabled()` usages to the
canonical `Input` rotation gate during this breaking-change slice, or provide a
strict one-way compatibility migration that serializes back to only the new
canonical representation.

## Mount input handoff

For this bounded slice, `AttachmentSystem` may continue to perform the current
snapshot/suspend/restore transaction, but it must operate only on translation:

1. Snapshot `InputComponent.translation_enabled`.
2. Set `translation_enabled = false` after mount commit.
3. Leave `enabled` and `rotation_enabled` unchanged.
4. Restore the exact captured translation value on normal dismount, forced
   unwind, target removal, and every rollback path.

This is an interim ownership location.  The separate movement-authority task
will later move the lease out of `AttachmentSystem`; the capability contract
introduced here remains valid when that migration occurs.

Restoring translation while a movement key is still physically held must have
a deterministic policy.  For this slice, require a fresh movement-key press
after restoration so a key held for vehicle control cannot immediately move
the dismounted Rider.

## FPS/look continuity

Mounting must not disable, discard, duplicate, or independently reconstruct the
desktop Input's retained FPS yaw/pitch state.  The base movement-root yaw and
the local look rotation have separate ownership.

After mount alignment:

- mouse and arrow deltas continue through the same FPS state;
- pitch remains clamped by the existing input behavior;
- yaw remains wrapped by the existing input behavior;
- the first look input does not restore a pre-mount world heading or jump;
- focused UI controls continue to reserve arrow keys.

If establishing the authored base yaw requires coordination with the input
pose driver, add an explicit synchronization boundary.  Attachment code must
not reach into `InputSystem`'s private FPS map.

## Implementation outline

1. Add canonical master/translation/rotation gates to `InputComponent`, its
   serialization, MMS registry, component documentation, and live API.
2. Make `InputSystem` evaluate translation and rotation independently while
   continuing to mutate only its existing direct controlled Transform.
3. Migrate the existing mode-owned rotation-disable configuration without
   retaining two writable authorities.
4. Change desktop mount suspension to snapshot and disable translation only;
   preserve the existing XR locomotion-only handoff.
5. Update horizontal alignment to account for the Rider-side mount point's
   relative yaw for desktop virtual look instead of assuming its orientation
   is identity.  Keep `left_display_car_desktop_mount` authored as the desired
   mounted frame and preserve the current XR live-head rule.
6. Add focused unit/integration coverage for mount alignment, retained look,
   exact restore, held-key reset, and mount cleanup.
7. Manually exercise mount, look, vehicle movement observation, and dismount in
   `mittens-corp-desktop.mms`.

## Acceptance criteria

- Unmounted desktop Input preserves existing WASD/R/F, mouse, arrow, and Q/E
  behavior.
- Mounting from at least three initial desktop headings places the Rider at the
  car mount and establishes the heading authored by the destination mount
  Transform.
- No generic same/opposed conditional or hidden 180-degree correction exists;
  rotating either authored endpoint changes the solved Rider-root heading.
- WASD/R/F does not translate the Rider while mounted.
- Right-mouse and arrow look continues smoothly while mounted and affects only
  the Input's existing controlled Transform.
- Vehicle handlers retain normal keyboard event observation; arrow look does
  not implicitly become vehicle steering.
- Moving and rotating the car carries the mounted Rider through the current
  retained follow relationship while local look remains usable.
- Dismount restores the exact pre-mount translation capability and requires a
  fresh movement-key press after a held vehicle-control key.
- Master-disabled and rotation-disabled Inputs remain authoritative throughout
  mount and dismount.
- Text fields and focused UI controls continue to capture arrow input.
- XR tracked pose, raw gamepad observation, and locomotion-only suspension are
  unchanged.

## Deferred

- first-class `MountPointComponent` materialization;
- attachment-valence validation and generic retained edges;
- configurable ongoing position/yaw/full-rotation constraints;
- generic movement-authority stacks and nested mounts;
- arbitrary constraint solvers, physics joints, wearables, and held-item
  migration;
- vehicle physics or a general vehicle controller.

## Related work

- [Vehicle mounting disables desktop look together with locomotion](../bugs/vehicle-mount-disables-desktop-look-with-locomotion.md)
- [`mittens-corp-desktop` mounted facing is reversed](../bugs/mittens-corp-desktop-mounted-facing-is-reversed.md)
- [Default desktop Input arrow-key camera look](desktop-input-default-arrow-camera-look.md)
- [Separate mounting from movement authority and input routing](attachment-movement-authority-and-input-routing.md)
- [Attachment valence and Grabbable unification](attachment-valence-and-grabbable-unification.md)
- [Rider + Mountable attachment-system first slice](rider-mountable-attachment-system-first-slice.md)
