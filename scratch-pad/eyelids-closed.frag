#ifdef GL_ES
precision mediump float;
#endif

uniform vec2 u_resolution;
uniform float u_time;

vec3 colorA = vec3(0.090,0.039,0.022);
vec3 colorB = vec3(0.9647, 0.4039, 0.0824);

void main() {
    vec3 color = vec3(0.0);

    float percent = ((sin(u_time / 2.5) / 3.0) + 0.5) + (cos(u_time * 20.0) / 30.0) + (sin(u_time * 7.0) / 30.0);
    //float percent = 0.0;

    // Mix uses pct (a value from 0-1) to
    // mix the two colors
    color = mix(colorA, colorB, percent);

    gl_FragColor = vec4(color,1.0);
}
