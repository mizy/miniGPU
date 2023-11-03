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

pub struct SpriteMaterial {
    pipeline: Option<wgpu::RenderPipeline>,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub shader_module: wgpu::ShaderModule,
    pub texture: texture::Texture,
    pub config: SpriteMaterialConfig,
    pub uniform_buffer: wgpu::Buffer,
}

pub struct SpriteMaterialConfig {
    pub width: f32,
    pub height: f32,
    pub radial: bool,
    pub size_attenuation: bool,
    pub shader: Option<String>,
    pub name: String,
    pub texture: Option<Texture>,
}
impl Default for SpriteMaterialConfig {
    fn default() -> Self {
        SpriteMaterialConfig {
            name: "Sprite".to_string(),
            width: 0.0,
            height: 0.0,
            radial: false,
            shader: None,
            texture: None,
            size_attenuation: true,
        }
    }
}

/// A material is a shader and its associated data.
/// use vs_main and fs_main as the entry points for the vertex and fragment shaders.

impl MaterialTrait for SpriteMaterial {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_name(&self) -> &str {
        "Sprite"
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

impl SpriteMaterial {
    pub fn new(mut config: SpriteMaterialConfig, renderer: &Renderer) -> SpriteMaterial {
        let device = &renderer.device;
        let shader_text = {
            if let Some(s) = config.shader.clone() {
                s
            } else {
                let mut shader_parser = ShaderParser::new();
                if !config.size_attenuation {
                    shader_parser
                        .defines
                        .insert("SIZE_ATTENUATION".to_string(), "true".to_string());
                }
                shader_parser
                    .parse_shader(include_str!("./shaders/sprite.wgsl"))
                    .to_string()
            }
        };
        println!("shader_text: {}", shader_text);
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: ShaderSource::Wgsl(shader_text.into()),
        });
        let texture = config.texture.unwrap();
        if config.width == 0. {
            config.width = texture.size.width as f32;
        }
        if config.height == 0. {
            config.height = texture.size.height as f32;
        }
        config.texture = None; // cause texture has been moved to texture

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // uniform config
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
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

        let uniform_buffer =
            Self::create_uniform_buffer(device, &[config.width as f32, config.height as f32, 0.]);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        SpriteMaterial {
            pipeline: None,
            shader_module,
            bind_group,
            bind_group_layout,
            uniform_buffer,
            texture,
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
