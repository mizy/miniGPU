@group(0) @binding(0) var<uniform> color: vec4f;

@vertex
fn vs_main(@location(0) pos: vec2f) ->@builtin(position) vec4f {
  return vec4f(pos, 0., 1.);
}

@fragment
fn fs_main() -> @location(0) vec4f {
  return color;
}