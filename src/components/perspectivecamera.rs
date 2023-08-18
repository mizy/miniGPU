use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::renderer::Renderer;
pub struct PerspectiveCamera {
    pub config: PerspectiveCameraConfig,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
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
            position: Vec3::new(0., 0., 1.),
            target: Vec3::new(0., 0., 0.),
            fov: 45.,
            aspect: 1.,
            near: 0.1,
            far: 100.,
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
        );
        let (bind_group, bind_group_layout) = Self::make_bind_group(uniform, renderer);
        let camera = PerspectiveCamera {
            config,
            bind_group,
            bind_group_layout,
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
        );
        let (bind_group, bind_group_layout) = Self::make_bind_group(uniform, &renderer);
        self.bind_group = bind_group;
        self.bind_group_layout = bind_group_layout;
    }

    fn make_bind_group(
        uniform: CameraUniform,
        renderer: &Renderer,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let device = &renderer.device;
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        (camera_bind_group, camera_bind_group_layout)
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.config.fov.to_radians(),
            self.config.aspect,
            self.config.near,
            self.config.far,
        )
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
    projection_matrix: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
}
impl CameraUniform {
    pub fn new(view: Mat4, projection: Mat4) -> Self {
        Self {
            view_projection: (projection * view).to_cols_array_2d(),
            projection_matrix: projection.to_cols_array_2d(),
            view_matrix: view.to_cols_array_2d(),
        }
    }
}
