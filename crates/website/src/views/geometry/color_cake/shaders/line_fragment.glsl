#version 300 es

precision highp float;

in vec3 out_color;
out vec4 fragColor;

void main() {
    fragColor = vec4(out_color, 0.0);
}

