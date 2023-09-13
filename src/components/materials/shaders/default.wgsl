@group(0) @binding(0) var<uniform> color: vec4f;

struct CameraUniform {
    view_projection: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
}
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(@location(0) pos: vec3f) ->@builtin(position) vec4f {
  return camera.view_projection * vec4f(pos, 1.);
}

@fragment
fn fs_main() -> @location(0) vec4f {
  return color;
}