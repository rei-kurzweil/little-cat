#version 450

layout(location = 0) in vec3 in_pos;

// Per-instance model matrix.
layout(location = 1) in vec4 i_model_c0;
layout(location = 2) in vec4 i_model_c1;
layout(location = 3) in vec4 i_model_c2;
layout(location = 4) in vec4 i_model_c3;

// Set 0: global camera.
layout(set = 0, binding = 0) uniform CameraUBO {
    mat4 view;
    mat4 proj;
    vec2 global_translation;
    vec2 _pad0;
} ubo;

layout(location = 0) out vec3 v_world_pos;
layout(location = 1) out vec3 v_normal;

void main() {
    mat4 model = mat4(i_model_c0, i_model_c1, i_model_c2, i_model_c3);

    vec4 world = model * vec4(in_pos, 1.0);
    // Camera pan: subtract camera position so the world moves opposite the camera.
    world.xy -= ubo.global_translation;

    // We don't have normals in the current vertex format, so use a cheap pseudo-normal.
    v_world_pos = world.xyz;
    v_normal = normalize(vec3(in_pos.xy, 1.0));

    gl_Position = ubo.proj * ubo.view * world;
}
