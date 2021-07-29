#ifdef GL_ES
precision mediump float;
#endif

#define PI 3.14159265359

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

vec3 colorA = vec3(0.298, 0.4392, 0.6745);
vec3 colorB = vec3(0.8784, 0.4902, 0.1686);

void main() {
    vec2 st = gl_FragCoord.xy/u_resolution.xy;
    vec3 color = vec3(0.0);

    vec3 pct = vec3(st.x);

    color = mix(colorA, colorB, pct);

    gl_FragColor = vec4(color,1.0);
}
