use crate::{renderer::Renderer, scene::Scene};

// ecs's system module
pub trait System {
    fn update(&self, renderer: &Renderer, scene: &Scene);
}
