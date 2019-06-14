#version 450

layout (location = 0) in vec4 position;
layout (location = 1) in vec4 color;

layout (binding = 0) uniform UBO
{
    mat4 viewProjection;
    mat4 world;
} ubo;

layout (location = 0) out vec4 o_color;

void main() {
    o_color = color;
    gl_Position = ubo.viewProjection * ubo.world * vec4(position.xyz, 1.0);
}