use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::renderer::Renderer;

use super::perspective_camera::{CameraTrait, CameraUniform};
pub struct OrthographicCamera {
    pub config: OrthographicCameraConfig,
    pub buffer: wgpu::Buffer,
    pub bind_index: u32,
}
pub struct OrthographicCameraConfig {
    pub width: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}
impl Default for OrthographicCameraConfig {
    fn default() -> Self {
        Self {
            width: 10.0,
            aspect: 1.0,
            near: 0.1,
            far: 100.0,
            position: Vec3::new(0.0, 0.0, 10.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
        }
    }
}

impl OrthographicCamera {
    pub fn new(config: OrthographicCameraConfig, renderer: &Renderer) -> OrthographicCamera {
        let uniform = CameraUniform::new(
            Mat4::look_at_rh(config.position, config.target, config.up),
            Mat4::orthographic_rh(
                -config.width / 2.0,
                config.width / 2.0,
                -config.width / 2.0 / config.aspect,
                config.width / 2.0 / config.aspect,
                config.near,
                config.far,
            ),
            config.position,
        );
        let buffer = Self::make_buffer(uniform, renderer);
        let camera = OrthographicCamera {
            config,
            bind_index: 0,
            buffer,
        };
        camera
    }

    fn make_buffer(uniform: CameraUniform, renderer: &Renderer) -> wgpu::Buffer {
        let device = &renderer.device;
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        camera_buffer
    }
}

impl CameraTrait for OrthographicCamera {
    fn update_bind_group(&mut self, renderer: &Renderer) {
        let uniform = CameraUniform::new(
            Mat4::look_at_rh(self.config.position, self.config.target, self.config.up),
            Mat4::orthographic_rh(
                -self.config.width / 2.0,
                self.config.width / 2.0,
                -self.config.width / 2.0 / self.config.aspect,
                self.config.width / 2.0 / self.config.aspect,
                self.config.near,
                self.config.far,
            ),
            self.config.position.clone(),
        );
        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]))
    }

    fn get_bind_index(&self) -> u32 {
        self.bind_index
    }

    fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn set_aspect(&mut self, aspect: f32, renderer: &Renderer) {
        self.config.aspect = aspect;
        self.update_bind_group(renderer);
    }

    fn get_type(&self) -> String {
        "orthographic".to_string()
    }
}
