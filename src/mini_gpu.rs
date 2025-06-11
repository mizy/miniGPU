use std::sync::Arc;

use winit::window::Window;

use crate::{
    components::camera::{
        camera_common::CameraType,
        perspective::{PerspectiveCamera, PerspectiveCameraConfig},
    },
    renderer::{Renderer, RendererConfig},
    system::{mesh_render::MeshRender, system::System, SystemManager},
    world,
};

pub struct MiniGPU {
    pub config: MiniGPUConfig,
    pub renderer: Renderer,
    pub world: world::World,
    pub system_manager: SystemManager,
}

pub struct MiniGPUConfig {
    pub width: u32,
    pub height: u32,
}

impl MiniGPU {
    pub async fn new(config: MiniGPUConfig, window: Window) -> MiniGPU {
        // add default world & renderer
        let renderer = Renderer::new(
            RendererConfig {
                width: config.width,
                height: config.height,
            },
            Arc::new(window),
        )
        .await;

        let world = world::World::new();

        let mut mini_gpu = MiniGPU {
            config,
            renderer,
            world,
            system_manager: SystemManager::new(),
        };
        mini_gpu
            .system_manager
            .add_system(Box::new(MeshRender::new()));

        // 自动添加默认实体
        mini_gpu.add_default_entities();

        mini_gpu
    }

    pub fn world_mut(&mut self) -> &mut world::World {
        &mut self.world
    }
    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn run(&mut self, delta_time: f32) {
        // 更新系统
        self.system_manager
            .run_systems(&mut self.world, &self.renderer, delta_time);
    }

    fn add_default_entities(&mut self) {
        self.add_default_camera();
    }

    /// 添加默认相机
    fn add_default_camera(&mut self) {
        let aspect = self.config.width as f32 / self.config.height as f32;

        let camera_config = PerspectiveCameraConfig::game_default()
            .with_aspect(aspect)
            .with_bind_index(0);

        let camera_entity = self.world.create_entity();
        let camera = PerspectiveCamera::from_config(camera_config);

        self.world.add_component(camera_entity, camera);
        self.world
            .set_main_camera(camera_entity, CameraType::Perspective);
    }

    pub fn cleanup(&mut self) {
        // 清理资源
        self.renderer.cleanup();
        self.world.cleanup();
        // 清理系统
        self.system_manager.cleanup();
    }
}
