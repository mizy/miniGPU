use glam::Vec3;

use crate::{components::perspective_camera::*, entity::Entity, renderer};
use std::any::Any;

#[derive(Default)]
pub struct Scene {
    pub entities: Vec<Entity>,
    pub background_color: wgpu::Color,
    pub default_camera: Option<usize>,
    components: Vec<*mut dyn Any>,
}
impl Scene {
    pub fn new() -> Scene {
        let instance = Scene {
            entities: Vec::new(),
            background_color: wgpu::Color::TRANSPARENT,
            components: Vec::new(),
            default_camera: None,
        };
        instance
    }

    pub fn add_default_entity(&mut self) -> usize {
        self.add_entity(Entity::new())
    }

    pub fn add_entity(&mut self, entity: Entity) -> usize {
        self.entities.push(entity);
        self.entities.len() - 1
    }

    pub fn add_entity_child(&mut self, parent_id: usize, mut entity: Entity) -> usize {
        entity.is_child = true;
        let child_id = self.add_entity(entity);
        let parent = self.entities.get_mut(parent_id);
        if parent.is_none() {
            panic!("parent entity not found");
        }
        let parent = parent.unwrap();
        parent.add_child(child_id);
        child_id
    }

    pub fn get_entity(&self, id: usize) -> &Entity {
        &self.entities[id]
    }
    pub fn get_entity_mut(&mut self, id: usize) -> &mut Entity {
        &mut self.entities[id]
    }

    pub fn remove_entity(&mut self, id: usize) {
        let entity = self.entities.remove(id);
        drop(entity)
    }

    // make component manually memory management
    pub fn add_component<T>(&mut self, component: T) -> usize {
        let b = Box::new(component);
        let raw_ptr = Box::into_raw(b);
        self.components.push(raw_ptr as *mut u8);
        self.components.len() - 1
    }

    pub fn set_entity_component<T>(&mut self, entity_id: usize, component: T, name: &str) -> usize {
        let component_ptr = self.add_component::<T>(component);
        let entity = self.entities.get_mut(entity_id).unwrap();
        entity.set_component_index(name, component_ptr);
        component_ptr
    }
    pub fn set_entity_component_index(
        &mut self,
        entity_id: usize,
        component_id: usize,
        name: &str,
    ) -> usize {
        let entity = self.entities.get_mut(entity_id).unwrap();
        entity.set_component_index(name, component_id);
        component_id
    }

    pub fn get_component<T>(&self, component_ptr: usize) -> &mut T {
        let addr = self.components.get(component_ptr).unwrap();
        let t = *addr as *mut T;
        unsafe { &mut *t }
    }

    pub fn drop_component<T>(&mut self, component_ptr: usize) {
        let addr = self.components.remove(component_ptr);
        drop(unsafe { Box::from_raw(addr as *mut T) });
    }

    pub fn get_entity_component<T>(&self, entity: &Entity, component: &str) -> &T {
        let component_ptr = entity.get_component_index(component);
        self.get_component::<T>(component_ptr)
    }

    pub fn get_entity_component_mut<T>(&self, entity: &Entity, component: &str) -> &mut T {
        let component_ptr = entity.get_component_index(component);
        self.get_component::<T>(component_ptr)
    }

    pub fn drop_entity_component<T>(&mut self, entity: &mut Entity, component: &str) {
        let component_ptr = entity.get_component_index(component);
        self.drop_component::<T>(component_ptr);
        entity.remove_component_index(component);
    }

    pub fn get_entity_component_index(&self, entity_id: usize, component_name: &str) -> usize {
        let entity = self.get_entity(entity_id);
        entity.get_component_index(component_name)
    }

    pub fn find_camera_entity(&self) -> Option<&Entity> {
        for entity in &self.entities {
            if entity.has_component("camera") {
                return Some(entity);
            }
        }
        None
    }

    pub fn get_default_camera(&self) -> Option<&mut Box<dyn CameraTrait>> {
        if self.default_camera.is_none() {
            return None;
        }
        let mut camera_entity: &Entity = self.get_entity(self.default_camera.unwrap());

        Some(self.get_entity_component_mut::<Box<dyn CameraTrait>>(camera_entity, "camera"))
    }

    pub fn add_default_camera(&mut self, renderer: &renderer::Renderer) {
        let entity_id = self.add_entity(Entity::new());
        let camera = PerspectiveCamera::new(
            PerspectiveCameraConfig {
                position: Vec3::new(0., 1., 1.),
                target: Vec3::new(0., 0., 0.),
                fov: 90.,
                aspect: renderer.config.width as f32 / renderer.config.height as f32,
                near: 0.001,
                far: 10000.,
                up: Vec3::new(0., 1., 0.),
            },
            renderer,
        );
        self.set_entity_component::<Box<dyn CameraTrait>>(entity_id, Box::new(camera), "camera");
        self.default_camera = Some(entity_id);
    }
}
