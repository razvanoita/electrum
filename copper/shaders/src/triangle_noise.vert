#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec3 color;

layout (set = 0, binding = 0) uniform UBView
{
    mat4 projection;
    mat4 view;
} ViewData;

layout (set = 0, binding = 1) uniform UBInstance
{
    mat4 world;
} InstanceData;

layout (location = 0) out vec4 o_color;
layout (location = 1) out vec3 o_normal_ws;
layout (location = 2) out vec3 o_position_ws;

void main() {
    o_color = vec4(color, 1.0);
    o_normal_ws = (InstanceData.world * vec4(normal.xyz, 0.0)).xyz;
    o_position_ws = (InstanceData.world * vec4(position.xyz, 1.0)).xyz;
    mat4 view_projection = ViewData.projection * ViewData.view; 
    gl_Position = view_projection * InstanceData.world  * vec4(position.xyz, 1.0);
}