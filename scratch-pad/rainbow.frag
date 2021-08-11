#ifdef GL_ES
precision highp float;
#endif

#define PI 3.1415926535897932384626433832795

uniform float u_time;
uniform vec2 u_resolution;

void main() {
  vec2 position = gl_FragCoord.xy / u_resolution.xy;
  vec3 color = vec3(0.0);

  color.r = sin(position.x*PI + PI/3.0);
  color.g = sin(position.x*PI);
  color.b = sin(position.x*PI - PI/3.0);


  gl_FragColor = vec4(color, 1.0);
}