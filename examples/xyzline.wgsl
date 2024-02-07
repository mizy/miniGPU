struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) position: vec3<f32>,
}
@vertex
fn vs_main(@location(0) pos: vec3f) -> VertexOutput {
  var out: VertexOutput;
  out.position = pos;
  out.clip_position = vec4f(pos, 1.);
  return out;
}

@fragment
fn fs_main(out:VertexOutput) -> @location(0) vec4f {
  return vec4f(out.position, 1.);
}