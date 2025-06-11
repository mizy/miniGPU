use crate::renderer::Renderer;
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

/// 相机基础数据 - 所有相机类型共享
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraData {
    /// 相机位置
    pub position: Vec3,
    /// 观察目标
    pub target: Vec3,
    /// 上方向向量
    pub up: Vec3,
    /// 近平面距离
    pub near: f32,
    /// 远平面距离
    pub far: f32,
    /// 在 shader 中的绑定索引
    pub bind_index: u32,
    /// 是否为主相机
    pub is_main: bool,
    pub dirty: bool, // 是否需要更新
}

impl CameraData {
    pub fn new(position: Vec3, target: Vec3, near: f32, far: f32, bind_index: u32) -> Self {
        Self {
            position,
            target,
            up: Vec3::Y,
            near,
            far,
            bind_index,
            is_main: false,
            dirty: true, // 默认创建时为脏数据
        }
    }

    /// 计算视图矩阵
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    pub fn update_dirty_flag(&mut self) {
        self.dirty = true;
    }
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 1.0, 1.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            near: 0.1,
            far: 1000.0,
            bind_index: 0,
            is_main: true,
            dirty: true, // 默认创建时为脏数据
        }
    }
}

/// GPU 使用的相机 Uniform 结构
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    /// 视图投影矩阵
    pub view_projection: [[f32; 4]; 4],
    /// 投影矩阵
    pub projection_matrix: [[f32; 4]; 4],
    /// 视图矩阵
    pub view_matrix: [[f32; 4]; 4],
    /// 相机位置
    pub position: [f32; 4],
}

impl CameraUniform {
    pub fn new(view: Mat4, projection: Mat4, position: Vec3) -> Self {
        Self {
            view_projection: (projection * view).to_cols_array_2d(),
            projection_matrix: projection.to_cols_array_2d(),
            view_matrix: view.to_cols_array_2d(),
            position: [position.x, position.y, position.z, 1.0],
        }
    }
}

/// 相机类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraType {
    Perspective,
    Orthographic,
}

/// 相机 GPU 资源组件（存储 GPU buffer）
pub struct CameraGpuData {
    pub buffer: wgpu::Buffer,
    pub camera_type: CameraType,
}

impl CameraGpuData {
    pub fn new(uniform: CameraUniform, camera_type: CameraType, renderer: &Renderer) -> Self {
        let device = &renderer.device;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            camera_type,
        }
    }

    pub fn update_buffer(&self, uniform: CameraUniform, renderer: &Renderer) {
        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}
