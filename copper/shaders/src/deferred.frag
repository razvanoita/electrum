#version 450

layout (binding = 0) uniform sampler2D i_normal_roughness_id;
layout (binding = 1) uniform sampler2D i_albedo_data;
layout (binding = 2) uniform sampler2D i_reflectance_ao;
layout (binding = 3) uniform sampler2D i_lighting;
layout (binding = 4) uniform sampler2D i_depth;

layout (location = 0) in vec2 i_uv;

layout (location = 0) out vec4 o_frag_color;

// --- http://jcgt.org/published/0003/02/01/paper.pdf
// --- Octahedron Vector Decoding
vec2 sign_not_zero(vec2 v) {
    return vec2(
        v.x >= 0.0 ? 1.0 : -1.0,
        v.y >= 0.0 ? 1.0 : -1.0
    );
}

vec3 octahedron_decoding(vec2 n) {
    vec3 v = vec3(n.xy, 1.0 - abs(n.x) - abs(n.y));
    if (v.z < 0.0) {
        v.xy = (1.0 - abs(v.yx)) * sign_not_zero(v.xy);
    }
    return normalize(v);
}

vec3 sRGB_to_linear(vec3 srgb) {
    vec3 res = vec3(0.0);
    if (srgb.x <= 0.04045 && srgb.y <= 0.04045 && srgb.z <= 0.04045) {
        res = srgb / 12.92;
    } else {
        res = pow((srgb + 0.055) / 1.055, vec3(2.4));
    }
    return res;
}

void main() {
    vec4 gbuffer0 = texture(i_normal_roughness_id, i_uv);
    vec3 normal_vs = octahedron_decoding(gbuffer0.xy);

	vec4 gbuffer1 = texture(i_albedo_data, i_uv);
	vec4 gbuffer2 = texture(i_reflectance_ao, i_uv);
    vec4 gbuffer3 = texture(i_lighting, i_uv);
    vec4 depth = texture(i_depth, i_uv);

    vec3 reflectance = sRGB_to_linear(gbuffer2.xyz);

    vec4 final_color = vec4(0.0);
    if (depth.r < 1.0) {
        final_color = vec4(gbuffer2.xyz, 1.0);
    } else {
        vec4 color0 = vec4(0.996, 0.349, 0.341, 1.0) * 0.3;
        vec4 color1 = vec4(0.984, 0.16, 0.337, 1.0) * 0.1;
        vec4 color2 = vec4(0.0);
        vec4 color3 = vec4(0.1);
        float factor = length(vec2(0.5) - i_uv) * 2.0;
        final_color = mix(
            mix(color0, color2, i_uv.x),
            mix(color1, color3, i_uv.y),
            factor
        );
    }

    o_frag_color = final_color;
}