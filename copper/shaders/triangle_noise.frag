#version 450

layout (location = 0) in vec4 o_color;
layout (location = 1) in vec3 o_normal_ws;
layout (location = 2) in vec3 o_position_ws;

layout (location = 0) out vec4 finalFragColor;

float rand(vec2 p) {
    p = 50.0 * fract(p * 0.3183099);
    return fract(p.x * p.y * (p.x + p.y));
}

float value_noise(in vec2 p) {
    vec2 X = floor(p);
    vec2 x = fract(p);
    
    vec2 fn = x * x * x * (6.0 * x * x - 15.0 * x + 10.0);
    float u = fn.x;
    float v = fn.y;
    
    float a = rand(X + vec2(0.0, 0.0));
    float b = rand(X + vec2(1.0, 0.0));
    float c = rand(X + vec2(0.0, 1.0));
    float d = rand(X + vec2(1.0, 1.0));
    
    float n = (b - a) * u + (c - a) * v + (a - b - c + d) * u * v + a;
    return 2.0 * n - 1.0;
}

void main() {
    vec3 normal_offset = vec3(
        value_noise(o_position_ws.xy * 100.0),
        value_noise(o_position_ws.yz * 100.0),
        value_noise(o_position_ws.xz * 100.0)
    ) * 0.5 + 0.5;
    float ndotl = clamp(
        dot(
            mix(o_normal_ws, normalize(o_normal_ws + normal_offset), 0.2), 
            -vec3(0.5)
        ),
        0.0, 
        1.0
    );

    finalFragColor = (o_color * ndotl);

}