use wgpu::util::DeviceExt;

use crate::renderer::Renderer;

use super::light::Light;

pub struct AmbientLight {
    pub uniform: AmbientLightUniform,
    pub buffer: wgpu::Buffer,
    pub bind_index: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AmbientLightUniform {
    pub color: [f32; 3],
    pub intensity: f32,
}

impl AmbientLight {
    pub fn new(renderer: &Renderer, index: u32, uniform: AmbientLightUniform) -> Self {
        let device = &renderer.device;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ambient Light Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            uniform,
            buffer,
            bind_index: index,
        }
    }

    pub fn update_buffer(&mut self, renderer: &Renderer) {
        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]))
    }
}

impl Light for AmbientLight {
    fn get_bind_index(&self) -> u32 {
        self.bind_index
    }

    fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
