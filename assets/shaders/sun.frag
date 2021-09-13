#version 450

layout(location=0)in float height;
layout(location=0)out vec4 o_Target;

void main(){
  o_Target=vec4(vec3(1.0, 0.9098, 0.4078),1.);
}
