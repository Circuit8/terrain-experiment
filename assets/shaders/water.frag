#version 450

layout(location=0) out vec4 o_Target;

layout(set=2,binding=0) uniform WaterMaterial_color {
  vec4 color;
};

layout(set=3,binding=0)uniform TimeUniform_value{
  float time;
};

void main(){
  o_Target = vec4(color.xyz, 0.85);
}
