#version 300 es
precision highp float;

in vec3 a_position;
in vec3 a_color;

out vec3 out_color;

uniform mat4 u_matrix;

void main() {
    out_color = a_color;
    gl_Position = u_matrix * vec4(a_position, 1.0);
}