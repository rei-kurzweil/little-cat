#version 450

// Position-only vertex format.
// Matches `Renderer::upload_mesh`, which packs `CpuVertex.pos` as 3x f32.
layout(location = 0) in vec3 in_pos;

// Per-draw push constants: model matrix (column-major).
layout(push_constant) uniform PushConstants {
    mat4 model;
} pc;

void main() {
    gl_Position = pc.model * vec4(in_pos, 1.0);
}
