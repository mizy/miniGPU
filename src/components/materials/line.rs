use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, BlendState, ColorTargetState, ShaderModuleDescriptor, ShaderSource};

use crate::{
    components::{material::MaterialTrait, mesh::Mesh},
    renderer::Renderer,
    utils::{
        depth_texture,
        texture::{self, Texture},
    },
};

use super::shader::ShaderParser;

pub struct LineMaterial {
    pipeline: Option<wgpu::RenderPipeline>,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub shader_module: wgpu::ShaderModule,
    pub config: LineMaterialConfig,
    pub uniform_buffer: wgpu::Buffer,
}

pub struct LineMaterialConfig {
    pub color: Vec<f32>,
    pub name: String,
    pub shader: Option<String>,
}
impl Default for LineMaterialConfig {
    fn default() -> Self {
        LineMaterialConfig {
            name: "Line".to_string(),
            color: vec![1.0, 1.0, 1.0, 1.0],
            shader: None,
        }
    }
}

/// A material is a shader and its associated data.
/// use vs_main and fs_main as the entry points for the vertex and fragment shaders.

impl MaterialTrait for LineMaterial {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_name(&self) -> &str {
        "Line"
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
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.shader_module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: renderer.swapchain_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            depth_stencil: Some(depth_texture::get_default_depth_stencil()),
        });
        println!("format: {:?}", renderer.swapchain_format);
        self.pipeline = Some(pipeline);
        self.pipeline.as_ref().unwrap()
    }
}

impl LineMaterial {
    pub fn new(mut config: LineMaterialConfig, renderer: &Renderer) -> LineMaterial {
        let device = &renderer.device;
        let shader_text = {
            if let Some(s) = config.shader.clone() {
                s
            } else {
                let mut shader_parser = ShaderParser::new();
                shader_parser
                    .parse_shader(include_str!("./shaders/default.wgsl"))
                    .to_string()
            }
        };
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: ShaderSource::Wgsl(shader_text.into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // uniform config
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let uniform_buffer = Self::create_uniform_buffer(device, config.color.as_slice());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
            label: Some("diffuse_bind_group"),
        });

        LineMaterial {
            pipeline: None,
            shader_module,
            bind_group,
            bind_group_layout,
            uniform_buffer,
            config,
        }
    }

    pub fn create_uniform_buffer(device: &wgpu::Device, uniforms: &[f32]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }
}
