#version 450

layout(location = 0) in vec3 in_pos;
layout(location = 5) in vec2 in_uv;

// Per-instance model matrix.
layout(location = 1) in vec4 i_model_c0;
layout(location = 2) in vec4 i_model_c1;
layout(location = 3) in vec4 i_model_c2;
layout(location = 4) in vec4 i_model_c3;
layout(location = 6) in vec4 i_color;

// Set 0: global camera.
// NOTE: This vertex shader currently applies `camera2d` + aspect correction before `proj*view`.
// It is intended for the current 2D camera path and is NOT Camera3D-ready yet.
layout(set = 0, binding = 0) uniform CameraUBO {
    mat4 view;
    mat4 proj;
    mat3 camera2d;
    vec2 viewport;
    vec2 _pad0;
} ubo;

layout(location = 0) out vec3 v_world_pos;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec2 v_uv;
layout(location = 3) out vec4 v_color;

void main() {
    mat4 model = mat4(i_model_c0, i_model_c1, i_model_c2, i_model_c3);

    vec4 world = model * vec4(in_pos, 1.0);

    // Apply 2D camera view transform (translation/scale/rotation).
    vec3 cam2d = ubo.camera2d * vec3(world.xy, 1.0);
    // Aspect-correct so 2D units are uniform on screen.
    float inv_aspect = (ubo.viewport.x > 0.0) ? (ubo.viewport.y / ubo.viewport.x) : 1.0;

    // World-space outputs (lighting expects world-space lights).
    v_world_pos = world.xyz;

    // Use a copy for the 2D camera/aspect-corrected clip transform.
    vec4 clip_world = world;
    clip_world.xy = vec2(cam2d.x * inv_aspect, cam2d.y);

    // Vertex format currently has no normals. For 2D primitives (XY plane), a stable forward
    // normal is +Z in object space; transform it into world space.
    v_normal = normalize(mat3(model) * vec3(0.0, 0.0, 1.0));
    v_uv = in_uv;
    v_color = i_color;

    gl_Position = ubo.proj * ubo.view * clip_world;
}
