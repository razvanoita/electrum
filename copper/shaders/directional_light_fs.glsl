#version 450

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_albedo;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput u_normals;

layout(push_constant) uniform PushConstants
{
    vec4 light_color;
    vec4 light_direction;
} push_constants;

layout(location = 0) out vec4 final_color;

void main()
{
    vec3 normal = normalize(subpassLoad(u_normals).xyz);
    float ndotl = max(dot(normal, push_constants.light_direction.xyz), 0.0);
    vec3 albedo = subpassLoad(u_albedo).rgb;
    final_color = vec4(ndotl * albedo * push_constants.light_color.rgb, 1.0);
}