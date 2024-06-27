use std::sync::Arc;

use winit::window::Window;

use crate::{
    renderer::{Renderer, RendererConfig},
    scene,
};

pub struct MiniGPU {
    pub config: MiniGPUConfig,
    pub renderer: Renderer,
    pub scene: scene::Scene,
}

pub struct MiniGPUConfig {
    pub width: u32,
    pub height: u32,
}

impl MiniGPU {
    pub async fn new(config: MiniGPUConfig, window: Window) -> MiniGPU {
        // add default scene & renderer
        let renderer = Renderer::new(
            RendererConfig {
                width: config.width,
                height: config.height,
            },
            Arc::new(window),
        )
        .await;
        let mut scene = scene::Scene::new();
        scene.add_default_camera(&renderer);
        MiniGPU {
            config,
            renderer,
            scene,
        }
    }

    pub fn get_default_scene(&mut self) -> &scene::Scene {
        &self.scene
    }
}
