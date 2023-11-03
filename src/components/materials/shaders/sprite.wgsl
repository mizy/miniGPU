// Vertex shader
struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) position: vec2<f32>,
    @location(2) radius: f32,
}

#include <CameraUniform>

@group(0) @binding(2)
var<uniform> size: vec3<f32>;


@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    var mvPosition = camera.view_matrix * vec4<f32>(input.position, 1.0);

    let m = f32(input.vertex_index%2u);
    let h = floor(f32(input.vertex_index) / 2.);
    let offset = vec2<f32>((m - 0.5) * size.x, -(h - 0.5) * size.y);
    mvPosition.x += offset.x; ;
    mvPosition.y += offset.y ;// y is inverted in webgpu
    
    #ifdef SIZE_ATTENUATION
    mvPosition.z = 0.;
    mvPosition.w = 1.;
    #endif

    out.radius = min(size.x, size.y)/2.;
    out.position = offset;
    out.clip_position = camera.projection_matrix * mvPosition;  // project to screen space
    out.tex_coord = vec2<f32>(m, h);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let distance = length(in.position) / in.radius;
    var color =  textureSample(t_diffuse, s_diffuse, in.tex_coord);
    //sdf 
    let alpha = 1.- smoothstep(0.95, 0.95 + 0.05, distance);
    if (alpha < 0.01) {
        discard;
    }
    color.a *= alpha;
    return color;
}