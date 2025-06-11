use crate::{renderer::Renderer, world::World};

// ecs's system module
pub trait System {
    fn update(&mut self, world: &mut World, delta_time: f32);
    fn render(&mut self, world: &mut World, renderer: &Renderer, delta_time: f32);
}
