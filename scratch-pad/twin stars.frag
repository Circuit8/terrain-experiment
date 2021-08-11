#ifdef GL_ES
precision highp float;
#endif

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

void main(){
  vec2 mouse_norm=u_mouse.xy/u_resolution;
  vec2 coord_norm=gl_FragCoord.xy/u_resolution;
  
  float g_adjustment=sin(u_time*4.)/8.;
  vec2 g_adjusted_coord=coord_norm+g_adjustment;
  float g_dist=abs(distance(mouse_norm,g_adjusted_coord));
  float g_intensity=(1.-g_dist)-.2;
  
  float r_adjustment=cos(u_time*4.)/8.;
  vec2 r_adjusted_coord=coord_norm-r_adjustment;
  float r_dist=abs(distance(mouse_norm,r_adjusted_coord));
  float r_intensity=(1.-r_dist)-.2;
  
  gl_FragColor=vec4(r_intensity,g_intensity,0.,1.);
}

// float color() {
  
// }