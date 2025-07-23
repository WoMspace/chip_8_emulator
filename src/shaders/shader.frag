#version 440

uniform layout(set = 3, binding = 0) vram {
    uvec4[16] video_memory;
};
uniform layout(set = 3, binding = 1) colors {
    vec3 foreground_color;
    vec3 background_color;
};

in vec4 gl_FragCoord;
const vec2 RESOLUTION = vec2(1280, 640);
const vec2 EMULATED_RESOLUTION = vec2(64.0, 32.0);
layout (location = 0) in vec4 v_color;
layout (location = 0) out vec4 frag_color;

void main() {
//    frag_color = v_color;
    frag_color = vec4(0.0, 0.0, 0.0, 1.0);
    ivec2 emulated_uv = ivec2(gl_FragCoord.xy) / 20;
    uvec4 quad = video_memory[emulated_uv.x / 4];
    uint column = 0;
    switch(emulated_uv.x % 4) {
            case 0: column = quad.x; break;
            case 1: column = quad.y; break;
            case 2: column = quad.z; break;
            case 3: column = quad.w; break;
    }
    
    bool pixel = ((column >> emulated_uv.y) & 1) == 1;
    
    frag_color.rgb = pixel ? foreground_color.rgb : background_color.rgb;
}