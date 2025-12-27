#version 450

layout(location = 0) out vec4 outColor;

void main() {
    vec2 p = gl_FragCoord.xy;

    // Repeat every N pixels (choose any size you like)
    float tile_px = 256.0;
    vec2 uv = fract(p / tile_px); // 0..1 repeating

    outColor = vec4(uv.x / 2.0, uv.y, uv.x+uv.y, 1.0);
}
