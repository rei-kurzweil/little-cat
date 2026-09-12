# `mittens-corp-desktop` preserves camera yaw when entering the vehicle

Status: open, reproduced by user on 2026-09-12.

## Observed behavior

In `examples/mittens-corp-desktop.mms`, approach the car while looking in a
direction different from the authored vehicle mount point and left-click the
car from inside its entry zone.  The Rider mounts and follows the vehicle, but
the desktop camera keeps its previous world-facing yaw instead of facing the
direction authored by the vehicle mount point.

Position attachment appears to commit.  The defect is specifically the
effective horizontal camera orientation after the mount transition.

## Expected behavior

On a successful desktop Rider-to-Mountable transition:

- the rider is placed at the vehicle's authored mount point;
- the effective desktop camera yaw is reset to the mount point's authored
  horizontal facing;
- no stale mouse-look/FPS yaw makes the camera preserve its pre-mount world
  direction;
- pitch behavior remains explicitly defined and independent from the yaw snap;
- repeated mount/dismount cycles produce the same facing result.

The desktop requirement is not automatically the XR requirement.  XR must
continue to distinguish locomotion/body yaw from the user's live tracked head
orientation; fixing desktop virtual look must not forcibly overwrite HMD pose.

## Current fixture

The desktop Rider references:

- movement root: `bisket_desktop_locomotion_root`;
- rider-side mount point: `bisket_first_person_camera_slot`;
- pedestrian input: `bisket_desktop_input`.

The car's `Mountable` references:

- eligibility zone: `left_display_car_front_zone`;
- destination mount point: `left_display_car_desktop_mount`;
- dismount point: `left_display_car_dismount`.

The destination point is an ordinary authored Transform today.  Its meaning
comes from `Mountable.mount_anchor(...)`; there is no first-class mount-point
component yet.

## Relevant implementation evidence

`AttachmentSystem::horizontal_anchor_alignment` already extracts yaw from the
destination mount transform and applies it to the Rider's outer movement root.
However, it aligns the rider-side point's position and deliberately does not
cancel unrestricted orientation below that root.

The desktop camera is below additional authored/runtime transforms:

```text
bisket_desktop_locomotion_root
  Input
    bisket_desktop_driver            // FPS/mouse yaw is retained here
      AVC / imported Bisket head
        bisket_first_person_camera_slot
          bisket_desktop_camera_rig
            Camera3D
```

`InputSystem` also retains FPS yaw/pitch/roll state keyed by the controlled
transform.  Changing only the outer locomotion root therefore may not make the
camera's effective world yaw equal the destination mount point yaw.  The exact
failure may involve the retained input transform, its cached FPS state, the
rider-side mount-point orientation, or a combination; capture all relevant
world transforms before choosing a fix.

## Mount-point terminology

Use **mount point** for the specific authored attachment endpoint whose
position and orientation participate in alignment.  A mount point is a kind
of socket in the broader terminology, but it is more precise for
Rider-to-Mountable placement.

Attachment valence belongs to mount points/endpoint roles, not to eligibility
zones.  A zone answers whether an attempted relation is allowed in a spatial
region; the selected mount points answer what attaches to what and how the two
frames align.

For the current car edge:

- the rider-side mount point has neutral/`0` valence;
- the vehicle destination mount point has positive/`+1` valence;
- the entry zone is referenced by that mounting policy but does not itself
  need an attachment valence.

## Investigation plan

1. Capture the world yaw of the locomotion root, desktop driver, rider-side
   mount point, destination mount point, and Camera3D immediately before and
   after mount.
2. Repeat after several nonzero mouse-yaw values and prove which transform or
   retained FPS value preserves the pre-mount heading.
3. Add a focused desktop test that mounts after rotating the driver and compares
   projected camera forward with projected destination mount-point forward.
4. Define a system boundary for resetting/synchronizing desktop look state when
   an attachment alignment intentionally changes facing.  Do not reach into an
   unrelated system's private cache.
5. Keep XR coverage proving that mount alignment changes locomotion/root yaw
   without destroying live tracked head orientation.

## Acceptance criteria

- Mounting from at least three different initial desktop yaws produces the same
  mount-point-directed camera yaw within a small tolerance.
- The rider-side and destination mount-point orientation offsets are honored;
  the test does not assume that either local transform is identity.
- The first mouse-look update after mounting continues smoothly from the
  snapped orientation and does not jump back to cached pre-mount yaw.
- Mounting does not introduce pitch or roll into the pedestrian locomotion
  root.
- Existing XR mount behavior and tracked-head orientation remain intact.
- Dismount/remount and vehicle motion preserve the authored mount-point facing
  contract.

## Related work

- [Vehicle mounting disables desktop look together with locomotion](vehicle-mount-disables-desktop-look-with-locomotion.md)
- [Attachment valence and Grabbable unification](../task/attachment-valence-and-grabbable-unification.md)
- [Rider + Mountable attachment-system first slice](../task/rider-mountable-attachment-system-first-slice.md)
- [Default desktop Input arrow-key camera look](../task/desktop-input-default-arrow-camera-look.md)
