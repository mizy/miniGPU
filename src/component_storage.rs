use std::{any::Any, collections::HashMap};

use crate::entity::Entity;

pub trait ComponentStorageTrait: Any {
    fn remove_entity(&mut self, entity: Entity) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct ComponentStorage<T> {
    components: Vec<T>,
    entities: Vec<Entity>,
    entity_to_index: HashMap<Entity, usize>,
}

impl<T: 'static> ComponentStorageTrait for ComponentStorage<T> {
    fn remove_entity(&mut self, entity: Entity) -> bool {
        self.remove(entity).is_some()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            entities: Vec::new(),
            entity_to_index: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, component: T) {
        if let Some(&index) = self.entity_to_index.get(&entity) {
            // 如果已存在，直接替换
            self.components[index] = component;
        } else {
            // 新增组件
            let index = self.components.len();
            self.components.push(component);
            self.entities.push(entity);
            self.entity_to_index.insert(entity, index);
        }
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.entity_to_index
            .get(&entity)
            .map(|&index| &self.components[index])
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.entity_to_index
            .get(&entity)
            .map(|&index| &mut self.components[index])
    }

    pub fn get_entities(&self) -> Vec<Entity> {
        self.entities.clone()
    }

    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let index = self.entity_to_index.remove(&entity)?;

        // 获取最后一个元素的信息（如果不是最后一个）
        let last_index = self.components.len() - 1;
        let moved_entity = if index != last_index {
            Some(self.entities[last_index])
        } else {
            None
        };

        // 移除组件和实体
        let removed_component = self.components.swap_remove(index);
        self.entities.swap_remove(index);

        // 如果有元素被移动，更新其索引
        if let Some(moved_entity) = moved_entity {
            self.entity_to_index.insert(moved_entity, index);
        }

        Some(removed_component)
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entity_to_index.contains_key(&entity)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.entities
            .iter()
            .zip(self.components.iter())
            .map(|(&e, c)| (e, c))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        self.entities
            .iter()
            .zip(self.components.iter_mut())
            .map(|(&e, c)| (e, c))
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}
