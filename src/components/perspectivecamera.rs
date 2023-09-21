use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::renderer::Renderer;
pub struct PerspectiveCamera {
    pub config: PerspectiveCameraConfig,
    pub buffer: wgpu::Buffer,
    pub bind_index: u32,
}
pub struct PerspectiveCameraConfig {
    pub position: Vec3,
    pub target: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub up: Vec3,
}
impl PerspectiveCameraConfig {
    pub fn default() -> Self {
        Self {
            position: Vec3::new(0., 1., 1.),
            target: Vec3::new(0., 0., 0.),
            fov: 90.,
            aspect: 1.,
            near: 0.1,
            far: 10000.,
            up: Vec3::new(0., 1., 0.),
        }
    }
}

impl PerspectiveCamera {
    pub fn new(config: PerspectiveCameraConfig, renderer: &Renderer) -> PerspectiveCamera {
        let uniform = CameraUniform::new(
            Mat4::look_at_rh(config.position, config.target, config.up),
            Mat4::perspective_rh(
                config.fov.to_radians(),
                config.aspect,
                config.near,
                config.far,
            ),
            config.position,
        );
        let buffer = Self::make_bind_group(uniform, renderer);
        let camera = PerspectiveCamera {
            config,
            bind_index: 0,
            buffer,
        };
        camera
    }

    pub fn update_bind_group(&mut self, renderer: &Renderer) {
        let uniform = CameraUniform::new(
            Mat4::look_at_rh(self.config.position, self.config.target, self.config.up),
            Mat4::perspective_rh(
                self.config.fov.to_radians(),
                self.config.aspect,
                self.config.near,
                self.config.far,
            ),
            self.config.position.clone(),
        );
        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]))
    }

    pub fn get_bind_index(&self) -> u32 {
        self.bind_index
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    fn make_bind_group(uniform: CameraUniform, renderer: &Renderer) -> wgpu::Buffer {
        let device = &renderer.device;
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        // let camera_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         label: Some("Camera Bind Group Layout"),
        //         entries: &[wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::VERTEX,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: None,
        //             },
        //             count: None,
        //         }],
        //     });
        // let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("Camera Bind Group"),
        //     layout: &camera_bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: camera_buffer.as_entire_binding(),
        //     }],
        // });
        camera_buffer
    }

    pub fn set_aspect(&mut self, aspect: f32, renderer: &Renderer) {
        self.config.aspect = aspect;
        self.update_bind_group(renderer);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
    projection_matrix: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    position: [f32; 4],
}
impl CameraUniform {
    pub fn new(view: Mat4, projection: Mat4, position: Vec3) -> Self {
        Self {
            view_projection: (projection * view).to_cols_array_2d(),
            projection_matrix: projection.to_cols_array_2d(),
            view_matrix: view.to_cols_array_2d(),
            position: [position.x, position.y, position.z, 1.],
        }
    }
}
