#version 450

layout (location = 0) in vec4 o_color;
layout (location = 1) in vec3 o_normal_ws;

layout (location = 0) out vec4 finalFragColor;

void main() {
    finalFragColor = o_color;
}