use std::{collections::HashMap, ptr};

use crate::{
    components::mesh::{self, Mesh},
    scene::Scene,
};

pub struct Entity {
    pub children: Vec<u32>,
    // map to scene's components
    pub components_map: HashMap<String, usize>,
}

impl Entity {
    pub fn new() -> Entity {
        let instance = Entity {
            children: Vec::new(),
            components_map: HashMap::new(),
        };
        instance
    }

    pub fn add_child(&mut self, child: u32) {
        self.children.push(child);
    }

    pub fn remove_child(&mut self, child: u32) {
        self.children.retain(|&x| x != child);
    }

    pub fn get_component_index(&self, component: &str) -> usize {
        *self.components_map.get(component).unwrap()
    }

    // make component manually memory management
    pub fn set_component_index(&mut self, name: &str, component_index: usize) {
        self.components_map
            .insert(name.to_string(), component_index);
    }

    pub fn remove_component_index(&mut self, name: &str) {
        self.components_map.remove(name).unwrap();
    }

    pub fn has_component(&self, component: &str) -> bool {
        self.components_map.contains_key(component)
    }
}
