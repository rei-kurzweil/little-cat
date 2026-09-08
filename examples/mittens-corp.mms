// mittens-corp — XR car/controller regression scene and Bisket pose-authoring tool.
//
// Run with:
//   cargo run --release -- load examples/mittens-corp.mms
//
// The vehicle rig deliberately gives AVC a non-humanoid model. Its XRHand
// children still own laser pointers, so controller pointing can be exercised
// even though the controlled model has no mapped hand bones.

import { tripod_light } from "../assets/components/tripod_light.mms"
import { truss } from "../assets/components/truss.mms"
import { bisket_anime_shading } from "../assets/components/materials/bisket_anime_shading.mms"
import { suspended_platform } from "../assets/components/platforms/suspended_platform.mms"

RendererSettings { window_size(1440, 810) }
BGC.rgba(0.055, 0.055, 0.060, 1.0)
AL.rgb(0.13, 0.13, 0.15)

RenderGraph {
    EmissivePass { BlurPass { radius_ndc(0.025) half_res(true) } }
    Bloom { intensity(0.42) radius_ndc(0.025) emissive_scale(1.0) half_res(true) }
}

fn stage_box(box_name, position, size, color) {
    return T.position(position[0], position[1], position[2])
        .scale(size[0], size[1], size[2]) {
        name = box_name
        R.cube() { C.rgba(color[0], color[1], color[2], 1.0) }
    }
}

// Keep the car example's studio stage so this tool also retains a stable
// lighting and mirror reference while XR transforms and gizmos are exercised.
stage_box(
    "studio_floor",
    [0.0, -0.92, 1.0],
    [54.0, 0.14, 32.0],
    [0.035, 0.037, 0.043],
)

stage_box("stage_deck",       [0.0,  0.00, -1.5], [32.0, 0.24, 14.0], [0.18, 0.18, 0.20])
stage_box("stage_upper_step", [0.0, -0.24,  5.7], [32.0, 0.28,  0.8], [0.14, 0.14, 0.16])
stage_box("stage_lower_step", [0.0, -0.56,  6.3], [32.0, 0.36,  0.8], [0.10, 0.10, 0.12])
stage_box("stage_back_wall",  [0.0,  4.00, -8.35], [32.0, 8.00, 0.35], [0.105, 0.105, 0.12])

T.position(0.0, 2.55, 8.10).scale(5.0, 5.0, 0.08).rotation(0.0, 3.1416, 0.0) {
    name = "stage_mirror"
    Grabbable {}
    R.cube() {
        Mirror.quality(2048) {}
        Raycastable.enabled()
    }
}

T.position(0.0, 7.10, -7.75) {
    name = "stage_ceiling_truss"
    truss(26)
}

// Three elevated walkway sections run along Z, perpendicular to the stage's
// long X axis. Their ends meet to form one continuous suspended platform.
for platform_index in range(3) {
    T.position(11.5, 4.0, (platform_index - 1) * 15.0) {
        suspended_platform()
    }
}

let subject_light_target = [0.0, 1.75, 1.7]
tripod_light(
    "front_left_studio_light",
    [-10.5, 0.14, 4.4],
    subject_light_target,
    SL.color(1.0, 0.82, 0.70).intensity(10.0).distance(22.0).angle(0.58).penumbra(0.32),
)
tripod_light(
    "front_right_studio_light",
    [10.5, 0.14, 4.4],
    subject_light_target,
    SL.color(0.72, 0.84, 1.0).intensity(10.0).distance(22.0).angle(0.58).penumbra(0.32),
)
tripod_light(
    "rear_left_studio_light",
    [-10.5, 0.14, -5.7],
    subject_light_target,
    SL.color(0.72, 0.84, 1.0).intensity(8.0).distance(20.0).angle(0.62).penumbra(0.38),
)
tripod_light(
    "rear_right_studio_light",
    [10.5, 0.14, -5.7],
    subject_light_target,
    SL.color(1.0, 0.78, 0.68).intensity(8.0).distance(20.0).angle(0.62).penumbra(0.38),
)

// XR replaces the desktop Input rig from car.mms. The outer transform is the
// locomotion target; InputXR supplies the tracked head pose to its child.
T.position(-5.0, 0.0, 0.0) {
    name = "car_locomotion_root"
    InputXR.on() {
        InputXRGamepad {
            locomotion()
            speed(4.0)
        }

        T {
            name = "car_xr_driver"
            AVC {
                name = "car_avatar_control"

                // AVC's first Transform child is intentionally a car rather
                // than a humanoid avatar.
                T.position(0.0, 0.10, 0.0) {
                    name = "car_vehicle"
                    T.rotation(0.0, 0.0, 0.0) {
                        name = "car_visual"
                        GLTF.new("assets/models/car.glb") {
                            bisket_anime_shading()
                        }
                    }
                }

                // Authored cockpit offset retained from car.mms; CXR supplies
                // the headset views and the camera pointer.
                T.position(0.0, 3.0, -1.55) {
                    name = "car_xr_cockpit_camera"
                    CXR { Pointer {} }
                }

                // These controllers intentionally do not attach to model hand
                // bones. Their tracked transforms must still drive pointers.
                XRHand.new(true, "Left", "GripAim").laser() {
                    T { Pointer {} }
                }
                XRHand.new(true, "Right", "GripAim").laser() {
                    T { Pointer {} }
                }
            }
        }
    }
}

// A second car sits beyond the left end of the stage. It is scenery only, so
// it does not interfere with the XR-controlled vehicle above.
T.position(-19.0, -0.75, -1.5).rotation(0.0, 0.30, 0.0) {
    name = "left_display_car"
    GLTF.new("assets/models/car.glb") {
        bisket_anime_shading()
    }
}

// Bisket is a separate editable pose target. Keeping it out of AVC prevents
// live XR IK from fighting authored gizmo rotations while poses are created.
ED.active() {
    T.position(4.0, 0.38, 0.0).rotation(0.0, 3.14159, 0.0) {
        name = "bisket_pose_subject"
        GLTF.new("assets/models/bisket.glb") {
            bisket_anime_shading()
            EM.on()
            PoseCapture { label("Bisket") asset_name("bisket") }
        }
    }
}

// Explicit selection disables every editor window except the two needed for
// pose-authoring and transform/gizmo diagnostics.
T.position(1.25, 2.8, -1.5) {
    name = "mittens_corp_editor_ui"
    EditorUI {
        panels([
            { panel = "settings" },
            { panel = "pose" },
        ])
    }
}

XR.on()
