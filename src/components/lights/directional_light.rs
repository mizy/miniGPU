use wgpu::util::DeviceExt;

use crate::renderer::Renderer;

use super::light::LightTrait;

pub struct DirectionalLight {
    pub uniform: DirectionalLightUniform,
    pub buffer: wgpu::Buffer,
    pub bind_index: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightUniform {
    pub direction: [f32; 3],
    pub color: [f32; 4],
    pub intensity: f32,
}

impl DirectionalLight {
    pub fn new(renderer: &Renderer, bind_index: u32, uniform: DirectionalLightUniform) -> Self {
        let device = &renderer.device;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Directional Light Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            uniform,
            buffer,
            bind_index,
        }
    }

    pub fn update_buffer(&mut self, renderer: &Renderer) {
        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]))
    }
}

impl LightTrait for DirectionalLight {
    fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    fn get_bind_index(&self) -> u32 {
        self.bind_index
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
