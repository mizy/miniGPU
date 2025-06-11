#include <CameraUniform>
#include <VertexStruct>

struct DirectionLight{
    direction: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
}

struct BlinnUniform {
    diffuse_color: vec3<f32>,
    diffuse_strength: f32,
    specular_color: vec3<f32>,
    specular_strength: f32,
    shininess: f32,
    opacity: f32,
    _padding: vec2<f32>, // 保持 16 字节对齐
};

@group(0) @binding(0) var<uniform> color: vec4f;
@group(1) @binding(1) var<uniform> direction_light: DirectionLight;
@vertex
fn vs_main(vertex:VertexInput) -> VertexOutput  {
  var out: VertexOutput;
  var clip_position = camera.projection_matrix * camera.view_matrix * vec4<f32>(vertex.position, 1.);
  out.clip_position = clip_position;
  out.position = vertex.position;
  out.normal = vertex.normal;
  #ifdef HAS_TEXTURE
  out.tex_coord = vertex.tex_coord; 
  #endif
  return out;
}


@group(0) @binding(0)
var diffuse_texture: texture_2d<f32>;
@group(0) @binding(1)
var diffuse_sampler: sampler;
@group(0) @binding(6) var<uniform> material: BlinnUniform;
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    #ifdef HAS_TEXTURE
    let color =  textureSample(diffuse_texture, diffuse_sampler, in.tex_coord);
    #else
    let color = vec4<f32>(material.diffuse_color, material.opacity);
    #endif

    //todo: not use hard code ambient_strength
    let ambient_strength = 0.1;
    let ambient_color = direction_light.color * ambient_strength;

    let light_dir = normalize(-direction_light.direction);
    let view_dir = normalize(camera.position.xyz - in.position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(in.normal, light_dir), 0.0);
    let diffuse_color = direction_light.color * diffuse_strength;

    let specular_strength = pow(max(dot(in.normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * direction_light.color;

    // specular_strength = pow(max(dot(in.normal, half_dir), 0.0), material.shininess) * material.specular_strength;
    // let specular_color = specular_strength * direction_light.color * material.specular_color;

    let result = (ambient_color + diffuse_color + specular_color) * color.rgb;

    return  vec4f(result, color.a);
}