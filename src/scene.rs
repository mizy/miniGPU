use std::collections::HashMap;

use crate::{
    entity::{self, Entity},
    system::system::System,
};

pub struct Scene {
    pub entities: Vec<Entity>,
    pub systems_map: HashMap<String, Box<dyn System>>,
    pub background_color: wgpu::Color,
    components: Vec<*mut u8>,
}
impl Scene {
    pub fn new() -> Scene {
        let instance = Scene {
            entities: Vec::new(),
            background_color: wgpu::Color::TRANSPARENT,
            systems_map: HashMap::new(),
            components: Vec::new(),
        };
        instance
    }
    pub fn add_entity(&mut self, entity: Entity) -> usize {
        self.entities.push(entity);
        self.entities.len() - 1
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

    pub fn add_system(&mut self, name: String, system: Box<dyn System>) -> Option<Box<dyn System>> {
        self.systems_map.insert(name, system)
    }

    pub fn remove_system(&mut self, name: &str) {
        let system = self.systems_map.remove(name).unwrap();
        drop(system);
    }
    // make component manually memory management
    pub fn add_component<T>(&mut self, component: T) -> usize {
        let layout = std::alloc::Layout::new::<T>();
        let ptr = unsafe {
            let ptr = std::alloc::alloc(layout) as *mut T;
            if ptr.is_null() {
                panic!("memory allocation failed");
            }
            std::ptr::write(ptr, component);
            ptr as *mut u8
        };
        self.components.push(ptr);
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

    pub fn get_component<T>(&self, component_ptr: usize) -> &T {
        let addr = self.components.get(component_ptr).unwrap();
        let t = *addr as *mut T;
        unsafe { &*t }
    }

    pub fn drop_component<T>(&mut self, component_ptr: usize) {
        let addr = self.components.remove(component_ptr);
        unsafe {
            let layout = std::alloc::Layout::new::<T>();
            std::alloc::dealloc(addr, layout);
        }
    }

    pub fn get_entity_component<T>(&self, entity: &Entity, component: &str) -> &T {
        let component_ptr = entity.get_component_index(component);
        self.get_component::<T>(component_ptr)
    }
    pub fn get_entity_component_index(&self, entity_id: usize, component_name: &str) -> usize {
        let entity = self.get_entity(entity_id);
        entity.get_component_index(component_name)
    }
}
