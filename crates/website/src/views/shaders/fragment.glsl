#version 300 es
precision highp float;

in vec3 out_color;
out vec4 fragColor;

uniform vec2 u_resolution;
uniform float u_hue;

vec3 hsl2rgb( in vec3 c )
{
    vec3 rgb = clamp( abs(mod(c.x*6.0+vec3(0.0,4.0,2.0),6.0)-3.0)-1.0, 0.0, 1.0 );

    return c.z + c.y * (rgb-0.5)*(1.0-abs(2.0*c.z-1.0));
}

void main() {
    vec2 relative_pos = vec2(gl_FragCoord[0], gl_FragCoord[1]) / u_resolution;
    float h = u_hue;
    float s = relative_pos[0];
    float l = relative_pos[1];
    vec3 color = hsl2rgb(vec3(h, s, l));
    fragColor = vec4(color, 0.0);
}

