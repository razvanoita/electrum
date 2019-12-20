#version 450

layout (location = 0) in vec4 o_color;
layout (location = 1) in vec3 o_normal_ws;
layout (location = 2) in vec3 o_position_ws;

layout (location = 0) out vec4 finalFragColor;

void main() {
    int counter = 8;
    int shift = int(o_position_ws.z * 16.0) & 3;
    int on = (counter >> shift) & 3;
    vec4 color0 = vec4(0.235, 0.258, 0.258, 1.0);
    vec4 color1 = vec4(0.984, 0.16, 0.337, 1.0);
    vec4 color2 = vec4(0.996, 0.349, 0.341, 1.0);
    float ndotl = clamp(dot(o_normal_ws, vec3(-0.5)), 0.0, 1.0);
    vec4 color = vec4(0.0, 0.0, 0.0, 1.0);
    float factor = float(on) / 2.0;
    if (factor == 1.0) {
        color = color1;
    } else if (factor == 0.0) {
        color = color0;
    } else {
        color = color2;
    }

    finalFragColor = color * ndotl;
}