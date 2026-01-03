#version 450

layout(location = 0) in vec3 v_world_pos;
layout(location = 1) in vec3 v_normal;

layout(location = 0) out vec4 f_color;

// Set 1: material params (no textures yet; those can be added later).
layout(set = 1, binding = 0) uniform MaterialUBO {
    vec4 base_color;
    vec3 light_dir_ws;
    float quant_steps;
    uint unlit;
    vec3 _pad0;
} mat;

float quantize(float x, float steps) {
    float s = max(1.0, steps);
    return floor(clamp(x, 0.0, 1.0) * s) / s;
}

void main() {
    vec3 base = mat.base_color.rgb;

    if (mat.unlit != 0u) {
        f_color = vec4(base, mat.base_color.a);
        return;
    }

    vec3 n = normalize(v_normal);
    vec3 l = normalize(mat.light_dir_ws);

    float ndotl = max(dot(n, l), 0.0);
    float q = quantize(ndotl, mat.quant_steps);

    vec3 lit = base * (0.15 + 0.85 * q);

    f_color = vec4(lit, mat.base_color.a);
}
