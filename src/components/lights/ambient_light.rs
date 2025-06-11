use super::light::LightData;

/// 环境光组件 - 纯数据结构
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmbientLight {
    /// 基础光照数据
    pub light_data: LightData,
}

/// GPU 使用的环境光数据结构
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AmbientLightUniform {
    /// 环境光颜色 (rgb) + 强度
    pub color_intensity: [f32; 4],
    /// 启用标志 + padding
    pub flags: [u32; 4],
}
impl AmbientLightUniform {
    pub fn default() -> Self {
        Self {
            color_intensity: [0.1, 0.1, 0.15, 0.4], // 默认环境光颜色和强度
            flags: [1, 0, 0, 0],                    // 启用标志
        }
    }
}

impl AmbientLight {
    /// 创建新的环境光组件
    pub fn new(color: [f32; 3], intensity: f32, bind_index: u32) -> Self {
        Self {
            light_data: LightData::new(color, intensity, bind_index),
        }
    }

    /// 创建默认的环境光
    pub fn default_ambient(bind_index: u32) -> Self {
        Self::new(
            [0.1, 0.1, 0.15], // 略带蓝色的环境光
            0.4,
            bind_index,
        )
    }
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self::default_ambient(0)
    }
}
