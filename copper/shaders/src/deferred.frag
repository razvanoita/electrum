#version 450

layout (binding = 0) uniform sampler2D i_normal_roughness_id;
layout (binding = 1) uniform sampler2D i_albedo_data;
layout (binding = 2) uniform sampler2D i_reflectance_ao;
layout (binding = 3) uniform sampler2D i_lighting;
layout (binding = 4) uniform sampler2D i_depth;

layout (location = 0) in vec2 i_uv;

layout (location = 0) out vec4 o_frag_color;


void main() {
    vec4 gbuffer0 = texture(i_normal_roughness_id, i_uv);
	vec4 gbuffer1 = texture(i_albedo_data, i_uv);
	vec4 gbuffer2 = texture(i_reflectance_ao, i_uv);
    vec4 gbuffer3 = texture(i_lighting, i_uv);
    float depth = texture(i_depth, i_uv).r;

    o_frag_color = gbuffer1;
}