use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, ShaderModuleDescriptor, ShaderSource};

use crate::{
    components::{material::MaterialTrait, mesh::Mesh},
    renderer::Renderer,
    utils::{
        depth_texture,
        texture::{self, Texture},
    },
};

pub struct BasicMaterial {
    pipeline: Option<wgpu::RenderPipeline>,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub shader_module: wgpu::ShaderModule,
    pub config: BasicMaterialConfig,
}

pub struct BasicMaterialConfig {
    pub width: u32,
    pub height: u32,
    pub shader: Option<String>,
    pub name: String,
    pub texture: Option<Texture>,
}
impl Default for BasicMaterialConfig {
    fn default() -> Self {
        BasicMaterialConfig {
            name: "basic".to_string(),
            width: 0,
            height: 0,
            shader: None,
            texture: None,
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

impl BasicMaterial {
    pub fn new(mut config: BasicMaterialConfig, renderer: &Renderer) -> BasicMaterial {
        let device = &renderer.device;
        let mut shader_text = include_str!("shaders/basic.wgsl");
        if let Some(text) = config.shader.as_ref() {
            shader_text = text
        }
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: ShaderSource::Wgsl(Cow::Borrowed(shader_text)),
        });
        if config.texture.is_none() {
            panic!("texture is none");
        }
        let texture = config.texture.unwrap();
        if config.width == 0 {
            config.width = texture.size.width;
        }
        if config.height == 0 {
            config.height = texture.size.height;
        }
        config.height = texture.size.height;
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
            ],
            label: Some("texture_bind_group_layout"),
        });
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
            ],
            label: Some("diffuse_bind_group"),
        });

        BasicMaterial {
            pipeline: None,
            shader_module,
            bind_group,
            bind_group_layout,
            config,
        }
    }

    pub fn make_image_mesh(
        &self,
        mut width: f32,
        mut height: f32,
        position: Vec<f32>,
        renderer: &Renderer,
    ) -> Mesh {
        #[repr(C)]
        #[derive(Clone, Copy, Pod, Zeroable)]
        struct Vertex {
            position: [f32; 3],
            tex_coord: [f32; 2],
        }
        width = width / 2.;
        height = height / 2.;
        //
        // 0----->1
        // |
        // |
        // |
        // 1
        let vertices = vec![
            Vertex {
                position: [position[0] - width, position[1] - height, position[2]],
                tex_coord: [0., 1.],
            },
            Vertex {
                position: [position[0] + width, position[1] - height, position[2]],
                tex_coord: [1., 1.],
            },
            Vertex {
                position: [position[0] + width, position[1] + height, position[2]],
                tex_coord: [1., 0.0],
            },
            Vertex {
                position: [position[0] - width, position[1] + height, position[2]],
                tex_coord: [0., 0.0],
            },
        ];
        let indices = vec![0, 1, 2, 2, 3, 0];
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let mesh = Mesh {
            vertex_buffer,
            index_buffer,
            vertex_buffer_layout: wgpu::VertexBufferLayout {
                array_stride: 5 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: 3 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                ],
            },
            num_indices: indices.len() as u32,
        };
        mesh
    }
}
