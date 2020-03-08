#version 450

layout (location = 0) in vec4 i_color;
layout (location = 1) in vec3 i_normal_vs;
layout (location = 2) in vec3 i_position_ws;

layout (location = 0) out vec4 o_normal_roughness_id;
layout (location = 1) out vec4 o_albedo_data;
layout (location = 2) out vec4 o_reflectance_ao;
layout (location = 3) out vec4 o_lighting;

// --- http://jcgt.org/published/0003/02/01/paper.pdf
// --- Octahedron Vector Encoding
vec2 sign_not_zero(vec2 v) {
    return vec2(
        v.x >= 0.0 ? 1.0 : -1.0,
        v.y >= 0.0 ? 1.0 : -1.0
    );
}

vec2 octahedron_encoding(vec3 v) {
    vec2 p = v.xy / (abs(v.x) + abs(v.y) + abs(v.z));
    return (v.z <= 0.0) ? (1.0 - abs(p.yx)) * sign_not_zero(p) : p;
}

void main() {
    vec2 encoded_normal_vs = octahedron_encoding(i_normal_vs.xyz);
    o_normal_roughness_id = vec4(encoded_normal_vs, 1.0, 1.0);

    int counter = 8;
    int shift = int(i_position_ws.y * 64.0) & 15;
    int on = (counter >> shift) & 1;
    vec4 color0 = vec4(0.235, 0.258, 0.258, 1.0);
    vec4 color1 = vec4(0.984, 0.16, 0.337, 1.0);
    o_albedo_data = vec4(mix(color0, color1, float(on)));

    o_reflectance_ao = vec4(i_color);

    o_lighting = vec4(1.0);
}