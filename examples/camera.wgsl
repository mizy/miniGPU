struct CameraUniform {
    view_projection: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec4f,
}
@group(0) @binding(0) var<uniform> color: vec4f;
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(@location(0) pos: vec3f) -> VertexOutput  {
  var out: VertexOutput;
  var clip_position = camera.projection_matrix * camera.view_matrix * vec4f(pos, 1.);
  out.clip_position = clip_position;
  out.color = vec4f(clip_position.xyz,1.);
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
  return in.color;
}