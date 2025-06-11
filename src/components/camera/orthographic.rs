use super::camera_common::*;
use glam::{Mat4, Vec3};

/// 正交相机配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrthographicCameraConfig {
    /// 相机位置
    pub position: Vec3,
    /// 目标位置
    pub target: Vec3,
    /// 视口宽度
    pub width: f32,
    /// 宽高比
    pub aspect: f32,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
    /// 缩放级别
    pub zoom: f32,
    /// 绑定索引
    pub bind_index: u32,
}

impl OrthographicCameraConfig {
    /// 创建新的正交相机配置
    pub fn new(position: Vec3, target: Vec3, width: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            position,
            target,
            width,
            aspect,
            near,
            far,
            zoom: 1.0,
            bind_index: 0,
        }
    }

    /// 设置缩放级别
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    /// 设置绑定索引
    pub fn with_bind_index(mut self, bind_index: u32) -> Self {
        self.bind_index = bind_index;
        self
    }

    /// 设置位置
    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    /// 设置目标
    pub fn with_target(mut self, target: Vec3) -> Self {
        self.target = target;
        self
    }

    /// 设置宽高比
    pub fn with_aspect(mut self, aspect: f32) -> Self {
        self.aspect = aspect;
        self
    }

    /// 创建默认的正交相机配置
    pub fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 10.0),
            target: Vec3::ZERO,
            width: 10.0,
            aspect: 1.0,
            near: 0.1,
            far: 1000.0,
            zoom: 1.0,
            bind_index: 0,
        }
    }

    /// 创建 2D 场景的默认配置
    pub fn default_2d() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 1.0),
            target: Vec3::ZERO,
            width: 20.0,
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
            zoom: 1.0,
            bind_index: 0,
        }
    }

    /// 创建适合UI的配置
    pub fn ui_config(screen_width: f32, screen_height: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 1.0),
            target: Vec3::ZERO,
            width: screen_width,
            aspect: screen_width / screen_height,
            near: -1.0,
            far: 1.0,
            zoom: 1.0,
            bind_index: 0,
        }
    }
}

impl Default for OrthographicCamera {
    fn default() -> Self {
        Self::from_config(OrthographicCameraConfig::default()) // 这里调用自定义的 default()
    }
}

/// 正交相机组件 - 纯数据结构
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrthographicCamera {
    /// 基础相机数据
    pub camera_data: CameraData,
    /// 视口宽度
    pub width: f32,
    /// 宽高比
    pub aspect: f32,
    /// 缩放级别
    pub zoom: f32,
}

impl OrthographicCamera {
    /// 从配置创建正交相机
    pub fn from_config(config: OrthographicCameraConfig) -> Self {
        Self {
            camera_data: CameraData::new(
                config.position,
                config.target,
                config.near,
                config.far,
                config.bind_index,
            ),
            width: config.width,
            aspect: config.aspect,
            zoom: config.zoom,
        }
    }

    /// 创建新的正交相机（保留原始方法作为便利函数）
    pub fn new(
        position: Vec3,
        target: Vec3,
        width: f32,
        aspect: f32,
        near: f32,
        far: f32,
        bind_index: u32,
    ) -> Self {
        let config = OrthographicCameraConfig::new(position, target, width, aspect, near, far)
            .with_bind_index(bind_index);
        Self::from_config(config)
    }

    /// 计算投影矩阵
    pub fn projection_matrix(&self) -> Mat4 {
        let width = self.width * self.zoom;
        let height = width / self.aspect;
        Mat4::orthographic_rh(
            -width / 2.0,
            width / 2.0,
            -height / 2.0,
            height / 2.0,
            self.camera_data.near,
            self.camera_data.far,
        )
    }

    /// 生成相机 Uniform 数据
    pub fn to_uniform(&self) -> CameraUniform {
        let view = self.camera_data.view_matrix();
        let projection = self.projection_matrix();
        CameraUniform::new(view, projection, self.camera_data.position)
    }

    /// 创建默认正交相机
    pub fn default_orthographic(bind_index: u32) -> Self {
        let config = OrthographicCameraConfig::default().with_bind_index(bind_index);
        Self::from_config(config)
    }

    /// 更新相机配置
    pub fn update_from_config(&mut self, config: OrthographicCameraConfig) {
        self.camera_data.position = config.position;
        self.camera_data.target = config.target;
        self.camera_data.near = config.near;
        self.camera_data.far = config.far;
        self.camera_data.bind_index = config.bind_index;
        self.width = config.width;
        self.aspect = config.aspect;
        self.zoom = config.zoom;
        self.camera_data.update_dirty_flag();
    }

    /// 获取当前配置
    pub fn to_config(&self) -> OrthographicCameraConfig {
        OrthographicCameraConfig {
            position: self.camera_data.position,
            target: self.camera_data.target,
            width: self.width,
            aspect: self.aspect,
            near: self.camera_data.near,
            far: self.camera_data.far,
            zoom: self.zoom,
            bind_index: self.camera_data.bind_index,
        }
    }
}
