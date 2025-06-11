@group(0) @binding(0) var<uniform> color: vec4f;

#include <CameraUniform>

@vertex
fn vs_main(@location(0) pos: vec3f) ->@builtin(position) vec4f {
  return camera.view_projection * vec4f(pos, 1.);
}

@fragment
fn fs_main() -> @location(0) vec4f {
  return color;
}