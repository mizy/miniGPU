struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    #ifdef HAS_TEXTURE
    @location(2) tex_coord: vec2<f32>,
    #endif
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    #ifdef HAS_TEXTURE
    @location(2) tex_coord: vec2<f32>,
    #endif
}