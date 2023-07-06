#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::{
    renderer::{self, Renderer, RendererConfig},
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
        let renderer = Renderer::new(
            RendererConfig {
                width: config.width,
                height: config.height,
            },
            window,
        )
        .await;
        MiniGPU {
            config,
            renderer,
            scene: scene::Scene::new(),
        }
    }
}
