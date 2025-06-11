/// 光照类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Ambient,
}

/// 基础光照数据结构 - 所有光照组件的通用字段
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightData {
    /// 光照颜色 (RGB)
    pub color: [f32; 3],
    /// 光照强度
    pub intensity: f32,
    /// 是否启用
    pub enabled: bool,
    /// 在 shader 中的绑定索引
    pub bind_index: u32,
}

impl LightData {
    pub fn new(color: [f32; 3], intensity: f32, bind_index: u32) -> Self {
        Self {
            color,
            intensity,
            enabled: true,
            bind_index,
        }
    }
}

impl Default for LightData {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            enabled: true,
            bind_index: 0,
        }
    }
}
