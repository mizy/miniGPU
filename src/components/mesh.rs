use wgpu::util::DeviceExt;

use crate::renderer::Renderer;

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
}

impl Mesh {
    pub fn new(vertices: Vec<f32>, indices: Vec<u32>, renderer: &Renderer) -> Mesh {
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
        let instance = Mesh {
            vertex_buffer,
            index_buffer,
            vertex_buffer_layout: wgpu::VertexBufferLayout {
                array_stride: 3 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                }],
            },
            num_indices: indices.len() as u32,
        };
        instance
    }

    pub fn set_data(&mut self, vertices: Vec<f32>, indices: Vec<u32>, renderer: &Renderer) {
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
        self.index_buffer = index_buffer;
        self.vertex_buffer = vertex_buffer;
    }

    pub fn get_buffer_layout(&self) -> wgpu::VertexBufferLayout {
        return self.vertex_buffer_layout.clone();
    }

    pub fn set_vertex_buffer_layout(
        &mut self,
        vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
    ) {
        self.vertex_buffer_layout = vertex_buffer_layout;
    }
}
