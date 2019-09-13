#version 450

layout (location = 0) in vec4 position;
layout (location = 1) in vec4 normal;
layout (location = 2) in vec4 color;

layout (set = 0, binding = 0) uniform UBO
{
    mat4 viewProjection;
    mat4 world;
} ubo;

layout (location = 0) out vec4 o_color;
layout (location = 1) out vec3 o_normal;

void main() {
    o_color = color;
    o_normal = (vec4(normal.xyz, 0.0)).xyz;
    gl_Position = ubo.viewProjection * ubo.world * vec4(position.xyz, 1.0);
}