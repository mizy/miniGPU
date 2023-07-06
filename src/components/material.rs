use std::borrow::Cow;
use wgpu::{util::DeviceExt, *};

use crate::renderer::Renderer;

pub struct Material {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub shader_module: wgpu::ShaderModule,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct MaterialConfig {
    pub shader_text: String,
    pub topology: wgpu::PrimitiveTopology,
    pub uniforms: Vec<f32>,
}

/// A material is a shader and its associated data.
/// use vs_main and fs_main as the entry points for the vertex and fragment shaders.
impl Material {
    pub fn new(config: MaterialConfig, renderer: &Renderer) -> Material {
        let device = &renderer.device;
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: ShaderSource::Wgsl(Cow::Borrowed(&config.shader_text)),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: 2 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(renderer.swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: config.topology, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        let bind_group = Material::create_bind_group(device, &config.uniforms, &bind_group_layout);
        Material {
            pipeline,
            shader_module,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn create_uniform_buffer(device: &wgpu::Device, uniforms: &[f32]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&uniforms),
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
