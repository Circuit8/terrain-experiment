#ifdef GL_ES
precision highp float;
#endif

uniform vec2 u_resolution;
vec3 black = vec3(0.0);
vec3 white = vec3(1.0);
vec3 red = vec3(0.8314, 0.2196, 0.0667);
vec3 blue = vec3(0.1608, 0.3569, 1.0);

void main() {
  float x = gl_FragCoord.x / u_resolution.x;
  vec3 color = black;

  float blue_mix = step(2.0/3.0, x);
  float white_mix = step(1.0/3.0, x) - step(2.0/3.0, x);
  float red_mix = step(0.0, x) - step(1.0/3.0, x);

  color = mix(color, blue, blue_mix);
  color = mix(color, red, red_mix);
  color = mix(color, white, white_mix);

  gl_FragColor = vec4(color, 1.0);
}