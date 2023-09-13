use std::mem;

use glam::Mat4;
use wgpu::{util::DeviceExt, VertexBufferLayout};

use crate::renderer::Renderer;

pub struct Instance {
    pub buffer: wgpu::Buffer,
    pub buffer_index: u32,
    pub data: Vec<InstanceData>,
    pub start_location: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub data: [[f32; 4]; 4],
}

impl Instance {
    pub fn new(data: Vec<InstanceData>, renderer: &Renderer) -> Self {
        Instance {
            buffer: Self::get_buffer(&data, &renderer.device),
            buffer_index: Self::get_buffer_index(),
            data,
            start_location: 5,
        }
    }

    pub fn get_buffer_index() -> u32 {
        1
    }

    pub fn get_buffer(instance_data: &Vec<InstanceData>, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    pub fn get_buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Mat4>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
