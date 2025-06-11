pub mod camera;
pub mod controller;
pub mod instance;
pub mod lights;
pub mod mesh_renderer;
pub mod transform;
pub mod viewport;

use std::any::Any;

/// 所有组件必须实现的基础 trait
pub trait Component: Any + Send + Sync {
    /// 获取组件的类型名称
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// 用于类型转换
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// 克隆组件（如果需要）
    fn clone_component(&self) -> Box<dyn Component>
    where
        Self: Clone,
    {
        Box::new(self.clone())
    }
}

/// 自动为所有实现了必要 trait 的类型实现 Component
impl<T: 'static + Send + Sync> Component for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
