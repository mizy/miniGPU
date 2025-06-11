use glam::Vec3;

use super::light::LightData;

/// 方向光组件 - 纯数据结构
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalLight {
    /// 基础光照数据
    pub light_data: LightData,
    /// 光照方向（世界空间）
    pub direction: Vec3,
    /// 是否投射阴影
    pub cast_shadows: bool,
}

/// GPU 使用的方向光数据结构
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightUniform {
    /// 光照方向 (xyz) + padding
    pub direction: [f32; 4],
    /// 光照颜色 (rgb) + 强度
    pub color_intensity: [f32; 4],
    /// 启用标志 + 阴影标志 + padding
    pub flags: [u32; 4],
}

impl DirectionalLightUniform {
    /// 创建新的方向光 uniform 数据
    pub fn default() -> Self {
        Self {
            direction: [0.0, -1.0, 0.0, 0.0],      // 默认方向向下
            color_intensity: [1.0, 1.0, 1.0, 3.0], // 白色光，强度为3
            flags: [1, 1, 0, 0],                   // 启用标志 + 阴影标志
        }
    }
}

impl DirectionalLight {
    /// 创建新的方向光组件
    pub fn new(direction: Vec3, color: [f32; 3], intensity: f32, bind_index: u32) -> Self {
        Self {
            light_data: LightData::new(color, intensity, bind_index),
            direction: direction.normalize(),
            cast_shadows: true,
        }
    }

    /// 创建默认的方向光（从上方照射的太阳光）
    pub fn default_sunlight(bind_index: u32) -> Self {
        Self::new(
            Vec3::new(-0.3, -1.0, -0.3),
            [1.0, 1.0, 0.9], // 略带暖色的白光
            3.0,
            bind_index,
        )
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self::default_sunlight(0)
    }
}
