#version 450

// Position-only vertex format.
// Matches the current mesh upload path (via `MeshUploader`), which packs `CpuVertex.pos` as 3x f32.
layout(location = 0) in vec3 in_pos;

// Per-instance model matrix, provided as 4 vec4 vertex attributes.
// These should come from a vertex input binding configured with
// VK_VERTEX_INPUT_RATE_INSTANCE.
layout(location = 1) in vec4 i_model_c0;
layout(location = 2) in vec4 i_model_c1;
layout(location = 3) in vec4 i_model_c2;
layout(location = 4) in vec4 i_model_c3;

// Uniform buffer: camera data comes from set=0,binding=0.
// For this debug mode we do *not* use view/proj yet; we use camera2d (mat3) as a 2D camera view transform.
// NOTE: This vertex shader is NOT Camera3D-ready yet. It ignores `view`/`proj` and emits clip-space-ish XY.
layout(set = 0, binding = 0) uniform CameraUBO {
    mat4 view;
    mat4 proj;
    mat3 camera2d;
    vec2 viewport;
    vec2 _pad0;
} ubo;

void main() {
    mat4 model = mat4(i_model_c0, i_model_c1, i_model_c2, i_model_c3);

    vec4 world = model * vec4(in_pos, 1.0);

    // Baseline clip-space projection just to see the scene.
    // NOTE: we intentionally ignore Z so objects at z=-2 remain visible.
    // Apply 2D camera view transform.
    vec3 cam2d = ubo.camera2d * vec3(world.xy, 1.0);
    // Aspect-correct clip-space so a unit in X matches a unit in Y.
    // Scale X by (height/width) so circles don't become ellipses.
    float inv_aspect = (ubo.viewport.x > 0.0) ? (ubo.viewport.y / ubo.viewport.x) : 1.0;
    gl_Position = vec4(cam2d.x * inv_aspect, cam2d.y, 0.0, 1.0);
}
