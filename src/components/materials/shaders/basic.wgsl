// basic.wgsl

// 相机绑定
struct CameraUniform {
    view_projection: mat4x4<f32>,
};
@group(1) @binding(0) var<uniform> camera: CameraUniform;

// 顶点输入/输出
struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec3<f32>,
    #ifdef HAS_TEXTURE
    @location(1) tex_coord: vec2<f32>,
    #endif
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

// 顶点着色器
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_projection * vec4<f32>(in.position, 1.0);
    
    #ifdef HAS_TEXTURE
        out.tex_coord = in.tex_coord;
    #else
        // 没有纹理坐标时生成默认坐标
        let x = f32(in.vertex_index % 2u);
        let y = f32((in.vertex_index / 2u) % 2u);
        out.tex_coord = vec2<f32>(x, y);
    #endif
    
    return out;
}


#ifdef HAS_TEXTURE
    // 纹理渲染模式
    @group(0) @binding(0) var t_diffuse: texture_2d<f32>;
    @group(0) @binding(1) var s_diffuse: sampler;
#else
    // 颜色渲染模式
    @group(0) @binding(0) var<uniform> color: vec4f;
#endif

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
#ifdef HAS_TEXTURE
      return textureSample(t_diffuse, s_diffuse, in.tex_coord);
#else
      return vec4f(color);
#endif
}