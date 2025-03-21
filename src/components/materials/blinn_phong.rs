use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::{util::DeviceExt, BindGroupEntry, ShaderModuleDescriptor, ShaderSource};

use crate::{
    components::material::MaterialTrait,
    renderer::Renderer,
    utils::{depth_texture, texture::Texture},
};

pub struct BlinnPhongMaterial {
    pipeline: Option<wgpu::RenderPipeline>,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub shader_module: wgpu::ShaderModule,
    pub config: BlinnPhongMaterialConfig,
}

/// 材质的 Uniform 数据
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MaterialUniform {
    pub diffuse_color: [f32; 3],
    pub diffuse_strength: f32,
    pub specular_color: [f32; 3],
    pub specular_strength: f32,
    pub shininess: f32,
    pub opacity: f32,
    // 填充数据以满足 16 字节对齐
    pub _padding: [f32; 2],
}

impl MaterialUniform {
    pub fn new(config: &BlinnPhongMaterialConfig) -> Self {
        Self {
            diffuse_color: config.diffuse_color.to_array(),
            diffuse_strength: config.diffuse_strength,
            specular_color: config.specular_color.to_array(),
            specular_strength: config.specular_strength,
            shininess: config.shininess,
            opacity: config.opacity,
            _padding: [0.0; 2],
        }
    }
}

pub struct BlinnPhongMaterialConfig {
    pub shader: Option<String>,
    /// 漫反射颜色
    pub diffuse_color: Vec3,

    /// 漫反射强度
    pub diffuse_strength: f32,

    /// 镜面反射颜色
    pub specular_color: Vec3,

    /// 镜面反射强度
    pub specular_strength: f32,

    /// 镜面高光指数（Shininess）
    pub shininess: f32,

    /// 环境光纹理（可选）
    pub ambient_texture: Option<Texture>,

    /// 漫反射纹理（可选）
    pub diffuse_texture: Option<Texture>,

    /// 镜面反射纹理（可选）
    pub specular_texture: Option<Texture>,

    /// 法线纹理（可选）
    pub normal_texture: Option<Texture>,

    /// 材质名称（用于调试或标识）
    pub name: String,

    /// 透明度（不透明度）
    pub opacity: f32,

    /// 材质 Uniform Buffer
    pub material_uniform_buffer: Option<wgpu::Buffer>,
}

impl Default for BlinnPhongMaterialConfig {
    fn default() -> Self {
        BlinnPhongMaterialConfig {
            diffuse_color: Vec3::new(1.0, 1.0, 1.0),
            diffuse_strength: 1.0,
            specular_color: Vec3::new(1.0, 1.0, 1.0),
            specular_strength: 1.0,
            shininess: 32.0,
            ambient_texture: None,
            diffuse_texture: None,
            specular_texture: None,
            normal_texture: None,
            name: "blinn_phong".to_string(),
            opacity: 1.0,
            shader: None,
            material_uniform_buffer: None,
        }
    }
}

/// A material is a shader and its associated data.
/// use vs_main and fs_main as the entry points for the vertex and fragment shaders.

impl MaterialTrait for BlinnPhongMaterial {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn get_name(&self) -> &str {
        "blinn_phong"
    }

    fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    fn get_render_pipeline(
        &mut self,
        renderer: &Renderer,
        env_pipeline_layout: &Vec<&wgpu::BindGroupLayout>,
        env_vertex_buffer_layout: Vec<wgpu::VertexBufferLayout>,
    ) -> &wgpu::RenderPipeline {
        if self.pipeline.is_some() {
            return self.pipeline.as_ref().unwrap();
        }
        let device = &renderer.device;
        let mut layouts = vec![&self.bind_group_layout];
        for layout in env_pipeline_layout {
            layouts.push(layout);
        }
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &layouts.as_slice(), //[env_pipeline_layout, layouts].concat().as_slice(),
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.shader_module,
                entry_point: "vs_main",
                buffers: &env_vertex_buffer_layout,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.shader_module,
                entry_point: "fs_main",
                targets: &[Some(renderer.swapchain_format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            depth_stencil: Some(depth_texture::get_default_depth_stencil()),
        });
        self.pipeline = Some(pipeline);
        self.pipeline.as_ref().unwrap()
    }
}

// 定义绑定索引常量
pub const BINDING_DIFFUSE_TEXTURE: u32 = 0;
pub const BINDING_DIFFUSE_SAMPLER: u32 = 1;
pub const BINDING_SPECULAR_TEXTURE: u32 = 2;
pub const BINDING_SPECULAR_SAMPLER: u32 = 3;
pub const BINDING_NORMAL_TEXTURE: u32 = 4;
pub const BINDING_NORMAL_SAMPLER: u32 = 5;
pub const BINDING_MATERIAL_UNIFORM: u32 = 6;

impl BlinnPhongMaterial {
    pub fn new(config: BlinnPhongMaterialConfig, renderer: &Renderer) -> BlinnPhongMaterial {
        let device = &renderer.device;
        let mut shader_text = include_str!("shaders/blinn_phong.wgsl");
        if let Some(text) = config.shader.as_ref() {
            shader_text = text
        }
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: ShaderSource::Wgsl(Cow::Borrowed(shader_text)),
        });

        let mut bind_groups: Vec<BindGroupEntry> = vec![];
        let mut bind_group_layouts: Vec<wgpu::BindGroupLayoutEntry> = vec![];

        // 处理漫反射纹理
        if let Some(diffuse_texture) = &config.diffuse_texture {
            Self::add_texture_and_sampler(
                &mut bind_groups,
                &mut bind_group_layouts,
                diffuse_texture,
                BINDING_DIFFUSE_TEXTURE,
                BINDING_DIFFUSE_SAMPLER,
            );
        }

        // 处理镜面反射纹理
        if let Some(specular_texture) = &config.specular_texture {
            Self::add_texture_and_sampler(
                &mut bind_groups,
                &mut bind_group_layouts,
                specular_texture,
                BINDING_SPECULAR_TEXTURE,
                BINDING_SPECULAR_SAMPLER,
            );
        }

        // 处理法线纹理
        if let Some(normal_texture) = &config.normal_texture {
            Self::add_texture_and_sampler(
                &mut bind_groups,
                &mut bind_group_layouts,
                normal_texture,
                BINDING_NORMAL_TEXTURE,
                BINDING_NORMAL_SAMPLER,
            );
        }

        // 创建 Uniform 数据
        let material_uniform = MaterialUniform::new(&config);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Uniform Buffer"),
            contents: bytemuck::cast_slice(&[material_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        bind_groups.push(BindGroupEntry {
            binding: BINDING_MATERIAL_UNIFORM,
            resource: uniform_buffer.as_entire_binding(),
        });

        bind_group_layouts.push(wgpu::BindGroupLayoutEntry {
            binding: BINDING_MATERIAL_UNIFORM,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        let bind_group_layout =
            Self::create_material_bind_group_layout(device, &bind_group_layouts);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &bind_groups,
            label: Some("diffuse_bind_group"),
        });

        BlinnPhongMaterial {
            pipeline: None,
            shader_module,
            bind_group,
            bind_group_layout,
            config,
        }
    }

    pub fn update_uniforms(&self, renderer: &Renderer) {
        let queue = &renderer.queue;
        let config = &self.config;
        if let Some(uniform_buffer) = &config.material_uniform_buffer {
            let material_uniform = MaterialUniform::new(config);
            queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[material_uniform]));
        }
    }

    pub fn create_material_bind_group_layout(
        device: &wgpu::Device,
        bind_group_layouts: &Vec<wgpu::BindGroupLayoutEntry>,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
            entries: &bind_group_layouts,
        })
    }

    // 辅助函数来添加绑定组
    pub fn add_texture_and_sampler<'a>(
        bind_groups: &mut Vec<BindGroupEntry<'a>>,
        bind_group_layouts: &mut Vec<wgpu::BindGroupLayoutEntry>,
        texture: &'a Texture,
        texture_binding: u32,
        sampler_binding: u32,
    ) {
        bind_groups.push(BindGroupEntry {
            binding: texture_binding,
            resource: wgpu::BindingResource::TextureView(&texture.view),
        });
        bind_groups.push(BindGroupEntry {
            binding: sampler_binding,
            resource: wgpu::BindingResource::Sampler(&texture.sampler),
        });
        bind_group_layouts.push(wgpu::BindGroupLayoutEntry {
            binding: texture_binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });
        bind_group_layouts.push(wgpu::BindGroupLayoutEntry {
            binding: sampler_binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
    }
}
