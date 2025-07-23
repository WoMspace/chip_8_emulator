#version 440
layout (location = 0) out vec4 v_color;
// glslc shader.vert -o shader.vert.spv
void main(void) {
    switch(gl_VertexIndex) {
        case 0: // bottom left
            gl_Position = vec4(-1.f, -1.f, 0.0f, 1.0f);
            v_color = vec4(1.f, 0.f, 0.f, 1.f);
            break;
        case 1: // bottom right
            gl_Position = vec4(1.0f, -1.0f, 0.0f, 1.0f);
            v_color = vec4(0.0f, 1.0f, 0.f, 1.f);
            break;
        case 2: // top right
            gl_Position = vec4(1.0f, 1.0f, 0.0f, 1.0f);
            v_color = vec4(0.0f, 0.0f, 1.f, 1.f);
            break;
        case 3: // top left
            gl_Position = vec4(-1.0f, 1.0f, 0.0f, 1.0f);
            v_color = vec4(1.0f, 0.0f, 1.0f, 1.0f);
            break;
        case 4: // bottom left
            gl_Position = vec4(-1.0f, -1.0f, 0.0f, 1.0f);
            v_color = vec4(1.0f, 0.0f, 0.0f, 1.0f);
            break;
    }
}