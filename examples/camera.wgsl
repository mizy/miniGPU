struct CameraUniform {
    view_projection: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
}
@group(0) @binding(0) var<uniform> color: vec4f;
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(@location(0) pos: vec2f) -> @builtin(position) vec4f {
  var clip_position = camera.projection_matrix * camera.view_matrix * vec4f(pos, 0., 1.);
  return clip_position;
}

@fragment
fn fs_main() -> @location(0) vec4f {
  return color;
}