#version 450

layout(location = 0) out vec4 outColor;

void main() {
    // Solid magenta to make it obvious it rendered.
    outColor = vec4(1.0, 0.0, 1.0, 1.0);
}