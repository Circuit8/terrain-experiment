#ifdef GL_ES
precision highp float;
#endif

#define PI 3.14159265359

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

vec3 colorSky = vec3(0.3137, 0.4706, 0.6196);
vec3 colorSea = vec3(0.0863, 0.2275, 0.4667);
vec3 colorSun = vec3(0.8784, 0.5451, 0.1686);
vec3 darkness = vec3(0.0,0.0,0.0);
float horizon = 0.333;

void main() {
    vec2 coords = gl_FragCoord.xy/u_resolution.xy;

    // get the horizon
    vec3 seaSkyMix = vec3(smoothstep(horizon -0.08, horizon, coords.y));
    vec3 color = mix(colorSky, colorSun, vec3(coords.x));
    vec3 sea = mix(colorSea, darkness, vec3(0.3 - coords.y));
    color = mix(sea, color, seaSkyMix);

    // mix the global day / night color
    float sunPosition = sin(u_time / 4.0) * 0.5 + 0.8;

    color = mix(darkness, color, vec3(sunPosition));

    
    gl_FragColor = vec4(color,1.0);
}
