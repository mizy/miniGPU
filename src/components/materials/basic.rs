use std::borrow::Cow;

use wgpu::{util::DeviceExt, ShaderModuleDescriptor, ShaderSource};

use crate::{
    components::material::MaterialTrait,
    renderer::Renderer,
    utils::{depth_texture, texture::Texture},
};

use super::shader::ShaderParser;
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct VertexFormatKey {
    pub has_texture: bool, // 材质是否使用纹理
}

pub struct BasicMaterial {
    pipeline: Option<wgpu::RenderPipeline>,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub shader_module: wgpu::ShaderModule,
    pub config: BasicMaterialConfig,
}

pub struct BasicMaterialConfig {
    pub shader: Option<String>,
    pub name: String,
    pub texture: Option<Texture>,
    pub color: [f32; 4],
}
impl Default for BasicMaterialConfig {
    fn default() -> Self {
        BasicMaterialConfig {
            name: "basic".to_string(),
            shader: None,
            texture: None,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// A material is a shader and its associated data.
/// use vs_main and fs_main as the entry points for the vertex and fragment shaders.

impl MaterialTrait for BasicMaterial {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn get_name(&self) -> &str {
        "image"
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
            label: Some("Basic Material Render Pipeline"),
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

impl BasicMaterial {
    pub fn new(mut config: BasicMaterialConfig, renderer: &Renderer) -> BasicMaterial {
        let device = &renderer.device;
        let has_texture = config.texture.is_some();
        let shader_text =
            Self::generate_shader_text(&config.shader, &VertexFormatKey { has_texture });

        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: ShaderSource::Wgsl(Cow::Borrowed(&shader_text)),
        });

        let bind_group_layout;
        let bind_group;
        if has_texture {
            // 纹理渲染模式
            let texture = config.texture.unwrap();
            config.texture = None; // 纹理已移出

            bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // 纹理
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
                    // 采样器
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

            bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                ],
                label: Some("texture_bind_group"),
            });
        } else {
            let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Color Uniform Buffer"),
                contents: bytemuck::cast_slice(&[config.color]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // 颜色 uniform
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("color_bind_group_layout"),
            });

            bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
                label: Some("color_bind_group"),
            });
        }

        BasicMaterial {
            pipeline: None,
            shader_module,
            bind_group,
            bind_group_layout,
            config,
        }
    }

    fn generate_shader_text(shader: &Option<String>, key: &VertexFormatKey) -> String {
        let mut shader_parser = ShaderParser::new();
        if key.has_texture {
            shader_parser
                .defines
                .insert("HAS_TEXTURE".to_string(), "true".to_string());
        }
        if let Some(shader) = shader {
            return shader_parser.parse_shader(shader).to_string();
        }

        shader_parser
            .parse_shader(include_str!("shaders/basic.wgsl"))
            .to_string()
    }
}
