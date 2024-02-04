pub mod mesh_line;
pub mod sprite_entity;

use std::collections::HashMap;

static mut ID: usize = 0;
pub struct Entity {
    pub name: String,
    pub id: usize,
    pub is_child: bool,
    pub children: Vec<usize>,
    // map to scene's components
    pub components_map: HashMap<String, usize>,
}

impl Entity {
    pub fn new() -> Entity {
        let instance = Entity {
            children: Vec::new(),
            name: String::from(""),
            id: unsafe {
                ID += 1;
                ID
            },
            is_child: false,
            components_map: HashMap::new(),
        };
        instance
    }

    pub fn add_child(&mut self, child: usize) -> usize {
        self.children.push(child);
        self.children.len() - 1
    }

    pub fn remove_child(&mut self, index: usize) {
        self.children.remove(index);
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
