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
// For this debug mode we do *not* use view/proj yet; we use global_translation in NDC.
layout(set = 0, binding = 0) uniform CameraUBO {
    mat4 view;
    mat4 proj;
    vec2 global_translation;
    vec2 _pad0;
} ubo;

void main() {
    mat4 model = mat4(i_model_c0, i_model_c1, i_model_c2, i_model_c3);

    vec4 world = model * vec4(in_pos, 1.0);

    // Baseline clip-space projection just to see the scene.
    // NOTE: we intentionally ignore Z so objects at z=-2 remain visible.
    // Apply global translation in NDC space.
    gl_Position = vec4(world.xy + ubo.global_translation, 0.0, 1.0);
}
