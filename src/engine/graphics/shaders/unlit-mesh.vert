#version 450

// Position-only vertex format.
// Matches `Renderer::upload_mesh`, which packs `CpuVertex.pos` as 3x f32.
layout(location = 0) in vec3 in_pos;

// Per-instance model matrix, provided as 4 vec4 vertex attributes.
// These should come from a vertex input binding configured with
// VK_VERTEX_INPUT_RATE_INSTANCE.
layout(location = 1) in vec4 i_model_c0;
layout(location = 2) in vec4 i_model_c1;
layout(location = 3) in vec4 i_model_c2;
layout(location = 4) in vec4 i_model_c3;

void main() {
    mat4 model = mat4(i_model_c0, i_model_c1, i_model_c2, i_model_c3);
    gl_Position = model * vec4(in_pos, 1.0);
}
