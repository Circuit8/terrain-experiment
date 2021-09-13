#version 450

layout(location=0)in float height;
layout(location=0)out vec4 o_Target;

vec3 baseTerrainColor(float height){
  if(height<1.){
    return vec3(.8,.7059,.1725);
  }else if(height<20.){
    return vec3(.2275,.8118,.2588);
  }else{
    return vec3(.6314,.6314,.6314);
  }
}

void main(){
  o_Target=vec4(baseTerrainColor(height),1.);
}
