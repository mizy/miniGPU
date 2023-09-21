struct CameraUniform {
    view_projection: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    position: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec4f,
    @location(1) position: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct DirectionLight{
    direction: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
}

@group(0) @binding(0) var<uniform> color: vec4f;
@group(1) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(1) var<uniform> direction_light: DirectionLight;


@vertex
fn vs_main(vertex:VertexInput) -> VertexOutput  {
  var out: VertexOutput;
  var clip_position = camera.projection_matrix * camera.view_matrix * vec4<f32>(vertex.position, 1.);
  out.clip_position = clip_position;
  out.position = vertex.position;
  out.normal = vertex.normal;
  out.color = vec4f(1.0,1.0,0.8, 1.0);
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let ambient_strength = 0.1;
    let ambient_color = direction_light.color * ambient_strength;

    let light_dir = normalize(-direction_light.direction);
    let view_dir = normalize(camera.position.xyz - in.position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(in.normal, light_dir), 0.0);
    let diffuse_color = direction_light.color * diffuse_strength;

    let specular_strength = pow(max(dot(in.normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * direction_light.color;

    let result = (ambient_color + diffuse_color + specular_color) * in.color.xyz;

    return  vec4f(result, in.color.a);
}