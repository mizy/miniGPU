use crate::{
    component_storage::{ComponentStorage, ComponentStorageTrait},
    components::camera::{
        camera_common::CameraType, orthographic::OrthographicCamera, perspective::PerspectiveCamera,
    },
    entity::Entity,
    resources::resource_manager::ResourceManager,
};
use std::{any::TypeId, collections::HashMap};

#[derive(Default)]
pub struct World {
    pub entities: Vec<Entity>,
    pub background_color: wgpu::Color,
    storages: HashMap<TypeId, Box<dyn ComponentStorageTrait>>,
    pub resource_manager: ResourceManager,
}
impl World {
    pub fn new() -> World {
        let instance = World {
            entities: Vec::new(),
            background_color: wgpu::Color::TRANSPARENT,
            storages: HashMap::new(),
            resource_manager: ResourceManager::new(),
        };
        instance
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity::new();
        self.entities.push(entity);
        entity
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        for storage in self.storages.values_mut() {
            storage.remove_entity(entity);
        }

        // 从实体列表中移除
        self.entities.retain(|&e| e != entity);
    }

    fn get_storage_mut<T: 'static>(&mut self) -> &mut ComponentStorage<T> {
        let type_id = TypeId::of::<T>();
        self.storages
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStorage::<T>::new()))
            .as_any_mut()
            .downcast_mut()
            .unwrap()
    }

    fn get_storage<T: 'static>(&self) -> Option<&ComponentStorage<T>> {
        self.storages
            .get(&TypeId::of::<T>())
            .and_then(|storage| storage.as_any().downcast_ref())
    }

    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        let storage = self.get_storage_mut::<T>();
        storage.insert(entity, component);
    }

    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.get_storage::<T>()?.get(entity)
    }

    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.get_storage_mut::<T>().get_mut(entity)
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Option<T> {
        self.get_storage_mut::<T>().remove(entity)
    }

    pub fn has_component<T: 'static>(&self, entity: Entity) -> bool {
        self.get_storage::<T>()
            .map(|storage| storage.contains(entity))
            .unwrap_or(false)
    }

    pub fn get_entities_with_component<T: 'static>(&self) -> Vec<Entity> {
        self.get_storage::<T>()
            .map(|storage| storage.get_entities())
            .unwrap_or_default()
    }

    /// 获取主相机实体
    pub fn get_main_camera_entity(&self) -> Option<(Entity, CameraType)> {
        // 查找透视相机中的主相机
        for entity in self.get_entities_with_component::<PerspectiveCamera>() {
            if let Some(camera) = self.get_component::<PerspectiveCamera>(entity) {
                if camera.camera_data.is_main {
                    return Some((entity, CameraType::Perspective));
                }
            }
        }

        // 查找正交相机中的主相机
        for entity in self.get_entities_with_component::<OrthographicCamera>() {
            if let Some(camera) = self.get_component::<OrthographicCamera>(entity) {
                if camera.camera_data.is_main {
                    return Some((entity, CameraType::Orthographic));
                }
            }
        }
        None
    }

    /// 设置主相机
    pub fn set_main_camera(&mut self, entity: Entity, camera_type: CameraType) {
        // 首先清除所有相机的主相机标志
        self.clear_main_camera_flags();

        // 设置指定相机为主相机
        match camera_type {
            CameraType::Perspective => {
                if let Some(camera) = self.get_component_mut::<PerspectiveCamera>(entity) {
                    camera.camera_data.is_main = true;
                }
            }
            CameraType::Orthographic => {
                if let Some(camera) = self.get_component_mut::<OrthographicCamera>(entity) {
                    camera.camera_data.is_main = true;
                }
            }
        }
    }

    fn clear_main_camera_flags(&mut self) {
        // 清除所有透视相机的主相机标志
        for entity in self.get_entities_with_component::<PerspectiveCamera>() {
            if let Some(camera) = self.get_component_mut::<PerspectiveCamera>(entity) {
                camera.camera_data.is_main = false;
            }
        }
        // 清除所有正交相机的主相机标志
        for entity in self.get_entities_with_component::<OrthographicCamera>() {
            if let Some(camera) = self.get_component_mut::<OrthographicCamera>(entity) {
                camera.camera_data.is_main = false;
            }
        }
    }

    pub fn cleanup(&mut self) {
        // 清理所有组件存储
        self.storages.clear();
        // 清理资源管理器
        self.resource_manager.cleanup();
    }
}
