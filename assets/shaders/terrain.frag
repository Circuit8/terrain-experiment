#version 450

layout(location=0)in float height;
layout(location=0)out vec4 o_Target;

float rand(vec3 coords){
  return fract(sin(dot(coords,vec3(12.9898,78.233, 54.02323)))*43758.5453);
}

vec3 baseTerrainColor(float height){
  if(height<1.){
    return vec3(.8,.7059,.1725) + ((rand(gl_FragCoord.xyz) - 0.5 ) / 5.0);
  } else {
    return vec3(.2275,.8118,.2588);
  }
}

void main(){
  vec3 color = baseTerrainColor(height);
  o_Target=vec4(color,1.);
}
