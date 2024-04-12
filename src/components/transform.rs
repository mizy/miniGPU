use glam::{Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;

use crate::renderer::Renderer;

pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
    pub matrix: glam::Mat4,
    pub global_matrix: glam::Mat4,
    pub buffer: wgpu::Buffer,
}

impl Transform {
    pub fn new(renderer: &Renderer, position: Vec3, rotation: Quat, scale: Vec3) -> Transform {
        let buffer = Self::make_buffer(Mat4::default(), renderer);
        Transform {
            position,
            rotation,
            scale,
            buffer,
            matrix: Mat4::from_scale_rotation_translation(scale, rotation, position),
            global_matrix: Mat4::default(),
        }
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.update_matrix();
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.update_matrix();
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.update_matrix();
    }

    pub fn update_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn update_global_matrix(&mut self, parent_global_matrix: glam::Mat4) {
        self.global_matrix = parent_global_matrix * self.matrix;
    }

    fn make_buffer(mat: Mat4, renderer: &Renderer) -> (wgpu::Buffer) {
        let device = &renderer.device;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&mat.to_cols_array()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        buffer
    }

    pub fn update_bind_group(&mut self, renderer: &Renderer) {}

    pub fn default(renderer: &Renderer) -> Transform {
        Transform::new(
            renderer,
            Vec3::new(0., 0., 0.),
            Quat::default(),
            Vec3::new(1., 1., 1.),
        )
    }
}
