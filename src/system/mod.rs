pub mod mesh_render;
pub mod system;

use crate::{renderer::Renderer, system::system::System, world::World};

// 创建独立的系统管理器
pub struct SystemManager {
    systems: Vec<Box<dyn System>>,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn run_systems(&mut self, world: &mut World, renderer: &Renderer, delta_time: f32) {
        // 更新阶段
        for system in &mut self.systems {
            system.update(world, delta_time);
        }

        // 渲染阶段
        for system in &mut self.systems {
            system.render(world, renderer, delta_time);
        }
    }

    pub fn cleanup(&mut self) {
        // 清理系统逻辑（如果需要）
        self.systems.clear();
    }
}
