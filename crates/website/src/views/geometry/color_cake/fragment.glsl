#version 300 es
precision highp float;

in vec3 out_color;
out vec4 fragColor;

uniform float u_hue;

vec3 hsl2rgb( in vec3 c )
{
    vec3 rgb = clamp( abs(mod(c.x*6.0+vec3(0.0,4.0,2.0),6.0)-3.0)-1.0, 0.0, 1.0 );

    return c.z + c.y * (rgb-0.5)*(1.0-abs(2.0*c.z-1.0));
}

void main() {
    float h = out_color[0] + u_hue;
    float s = out_color[1];
    float l = out_color[2];
    vec3 color = hsl2rgb(vec3(h, s, l));
    fragColor = vec4(color, 0.0);
}

