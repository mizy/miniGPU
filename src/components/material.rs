use std::borrow::Cow;
use wgpu::{util::DeviceExt, *};

use crate::{renderer::Renderer, utils::depth_texture};

pub struct Material {
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub bind_group: wgpu::BindGroup,
    pub shader_module: wgpu::ShaderModule,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub config: MaterialConfig,
}

pub struct MaterialConfig {
    pub shader: String,
    pub topology: wgpu::PrimitiveTopology,
    pub uniforms: Vec<f32>,
}

impl MaterialTrait for Material {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn get_name(&self) -> &str {
        "Material"
    }

    fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn get_render_pipeline(
        &mut self,
        renderer: &Renderer,
        env_pipeline_layout: &Vec<&BindGroupLayout>,
        env_vertex_buffer_layout: Vec<VertexBufferLayout>,
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
            label: Some("Render Pipeline Material"),
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
                targets: &[Some(ColorTargetState {
                    format: renderer.swapchain_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.config.topology, // 1.
                strip_index_format: { 
                    if self.config.topology == wgpu::PrimitiveTopology::TriangleStrip {
                        Some(wgpu::IndexFormat::Uint32)
                    } else {
                        None
                    }
                },
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(depth_texture::get_default_depth_stencil()),
            multisample: wgpu::MultisampleState{
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        self.pipeline = Some(pipeline);
        self.pipeline.as_ref().unwrap()
    }
}

/// A material is a shader and its associated data.
/// use vs_main and fs_main as the entry points for the vertex and fragment shaders.
impl Material {
    pub fn new(config: MaterialConfig, renderer: &Renderer) -> Material {
        let device = &renderer.device;
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: ShaderSource::Wgsl(Cow::Borrowed(&config.shader)),
        });
        let bind_group_layout = Material::create_bind_group_layout(device);
        let bind_group = Material::create_bind_group(device, &config.uniforms, &bind_group_layout);

        Material {
            config,
            pipeline: None,
            shader_module,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("fragment_bind_group_layout"),
        })
    }

    pub fn create_uniform_buffer(device: &wgpu::Device, uniforms: &[f32]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }

    pub fn create_bind_group(
        device: &wgpu::Device,
        uniforms: &Vec<f32>,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let uniform_buffer = Material::create_uniform_buffer(device, uniforms);
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        })
    }
}

pub trait MaterialTrait {
    fn get_name(&self) -> &str;
    fn get_bind_group(&self) -> &wgpu::BindGroup;
    fn get_render_pipeline(
        &mut self,
        renderer: &Renderer,
        env_pipeline_layout: &Vec<&BindGroupLayout>,
        env_vertex_buffer_layout: Vec<VertexBufferLayout>,
    ) -> &wgpu::RenderPipeline;
    fn as_any(&mut self) -> &mut dyn std::any::Any;
}
