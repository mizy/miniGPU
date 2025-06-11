use super::camera_common::*;
use glam::{Mat4, Vec3};

/// 透视相机配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveCameraConfig {
    /// 相机位置
    pub position: Vec3,
    /// 目标位置
    pub target: Vec3,
    /// 视野角度（弧度）
    pub fov: f32,
    /// 宽高比
    pub aspect: f32,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
    /// 绑定索引
    pub bind_index: u32,
}

impl PerspectiveCameraConfig {
    /// 创建新的透视相机配置
    pub fn new(position: Vec3, target: Vec3, fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            position,
            target,
            fov,
            aspect,
            near,
            far,
            bind_index: 0,
        }
    }

    /// 设置视野角度（度数）
    pub fn with_fov_degrees(mut self, fov_degrees: f32) -> Self {
        self.fov = fov_degrees.to_radians();
        self
    }

    /// 设置视野角度（弧度）
    pub fn with_fov_radians(mut self, fov_radians: f32) -> Self {
        self.fov = fov_radians;
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

    /// 设置裁剪面
    pub fn with_clipping(mut self, near: f32, far: f32) -> Self {
        self.near = near;
        self.far = far;
        self
    }

    /// 创建默认的透视相机配置
    pub fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 1.0, 3.0),
            target: Vec3::ZERO,
            fov: 45.0_f32.to_radians(), // 45度视野角
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            bind_index: 0,
        }
    }

    /// 创建适合游戏的默认配置
    pub fn game_default() -> Self {
        Self {
            position: Vec3::new(0.0, 2.0, 5.0),
            target: Vec3::ZERO,
            fov: 60.0_f32.to_radians(), // 60度视野角
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            bind_index: 0,
        }
    }

    /// 创建第一人称视角配置
    pub fn first_person() -> Self {
        Self {
            position: Vec3::new(0.0, 1.8, 0.0), // 人眼高度
            target: Vec3::new(0.0, 1.8, -1.0),
            fov: 75.0_f32.to_radians(), // 较宽的视野角
            aspect: 16.0 / 9.0,
            near: 0.01, // 更近的近裁剪面
            far: 1000.0,
            bind_index: 0,
        }
    }

    /// 创建第三人称视角配置
    pub fn third_person() -> Self {
        Self {
            position: Vec3::new(0.0, 3.0, 8.0),
            target: Vec3::ZERO,
            fov: 45.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            bind_index: 0,
        }
    }

    /// 创建建筑/场景展示配置
    pub fn architectural() -> Self {
        Self {
            position: Vec3::new(10.0, 10.0, 10.0),
            target: Vec3::ZERO,
            fov: 35.0_f32.to_radians(), // 较窄的视野角，减少透视失真
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 5000.0, // 更远的视距
            bind_index: 0,
        }
    }

    /// 创建摄影风格配置
    pub fn portrait() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 3.0),
            target: Vec3::ZERO,
            fov: 85.0_f32.to_radians(), // 85mm镜头等效
            aspect: 4.0 / 3.0,          // 传统照片比例
            near: 0.1,
            far: 100.0,
            bind_index: 0,
        }
    }

    /// 获取视野角度（度数）
    pub fn fov_degrees(&self) -> f32 {
        self.fov.to_degrees()
    }
}

/// 透视相机组件 - 纯数据结构
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveCamera {
    /// 基础相机数据
    pub camera_data: CameraData,
    /// 视野角度（弧度）
    pub fov: f32,
    /// 宽高比
    pub aspect: f32,
}

impl PerspectiveCamera {
    /// 从配置创建透视相机
    pub fn from_config(config: PerspectiveCameraConfig) -> Self {
        Self {
            camera_data: CameraData::new(
                config.position,
                config.target,
                config.near,
                config.far,
                config.bind_index,
            ),
            fov: config.fov,
            aspect: config.aspect,
        }
    }

    /// 创建新的透视相机（保留原始方法作为便利函数）
    pub fn new(
        position: Vec3,
        target: Vec3,
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
        bind_index: u32,
    ) -> Self {
        let config = PerspectiveCameraConfig::new(position, target, fov, aspect, near, far)
            .with_bind_index(bind_index);
        Self::from_config(config)
    }

    /// 计算投影矩阵
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.fov,
            self.aspect,
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

    /// 创建默认透视相机
    pub fn default_perspective(bind_index: u32) -> Self {
        let config = PerspectiveCameraConfig::default().with_bind_index(bind_index);
        Self::from_config(config)
    }

    /// 更新相机配置
    pub fn update_from_config(&mut self, config: PerspectiveCameraConfig) {
        self.camera_data.position = config.position;
        self.camera_data.target = config.target;
        self.camera_data.near = config.near;
        self.camera_data.far = config.far;
        self.camera_data.bind_index = config.bind_index;
        self.fov = config.fov;
        self.aspect = config.aspect;
        self.camera_data.update_dirty_flag();
    }

    /// 获取当前配置
    pub fn to_config(&self) -> PerspectiveCameraConfig {
        PerspectiveCameraConfig {
            position: self.camera_data.position,
            target: self.camera_data.target,
            fov: self.fov,
            aspect: self.aspect,
            near: self.camera_data.near,
            far: self.camera_data.far,
            bind_index: self.camera_data.bind_index,
        }
    }

    /// 获取视野角度（度数）
    pub fn fov_degrees(&self) -> f32 {
        self.fov.to_degrees()
    }

    /// 设置视野角度（度数）
    pub fn set_fov_degrees(&mut self, degrees: f32) {
        self.fov = degrees.to_radians();
        self.camera_data.update_dirty_flag();
    }

    /// 设置视野角度（弧度）
    pub fn set_fov_radians(&mut self, radians: f32) {
        self.fov = radians;
        self.camera_data.update_dirty_flag();
    }
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self::from_config(PerspectiveCameraConfig::default())
    }
}
