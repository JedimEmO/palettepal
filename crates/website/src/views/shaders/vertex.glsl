#version 300 es
precision highp float;

in vec2 a_position;
in vec3 a_color;

out vec3 out_color;

void main() {
    out_color = a_color;
    gl_Position = vec4(a_position, 0.0, 1.0);
}