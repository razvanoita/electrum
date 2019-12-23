#version 450

layout (location = 0) in vec4 o_color;
layout (location = 1) in vec3 o_normal_ws;
layout (location = 2) in vec3 o_position_ws;

layout (location = 0) out vec4 finalFragColor;

void main() {
    int counter = 8;
    int shift = int(o_position_ws.y * 64.0) & 15;
    int on = (counter >> shift) & 1;
    vec4 color0 = vec4(0.235, 0.258, 0.258, 1.0);
    vec4 color1 = vec4(0.984, 0.16, 0.337, 1.0);
    float ndotl = clamp(dot(o_normal_ws, vec3(-0.5)), 0.0, 1.0);

    finalFragColor = mix(color0, color1, float(on)) * ndotl;
}