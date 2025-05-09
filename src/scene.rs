use glam::Vec3;

use crate::{
    components::{
        lights::{
            directional_light::{DirectionalLight, DirectionalLightUniform},
            light::LightTrait,
        },
        perspective_camera::*,
    },
    entity::Entity,
    renderer,
};
use std::any::{Any, TypeId};
const DEFAULT_CAMERA_BIND_INDEX: u32 = 0;
const DEFAULT_LIGHT_BIND_INDEX: u32 = 1;

pub struct Component {
    ptr: *mut dyn Any,       // 指向组件实例的指针
    type_id: TypeId,         // 存储组件的类型 ID
    type_name: &'static str, // 存储类型名称，用于错误消息
}

#[derive(Default)]
pub struct Scene {
    pub entities: Vec<Entity>,
    pub background_color: wgpu::Color,
    pub default_camera: Option<usize>,
    pub default_light: Option<usize>,
    components: Vec<Component>,
}
impl Scene {
    pub fn new() -> Scene {
        let instance = Scene {
            entities: Vec::new(),
            background_color: wgpu::Color::TRANSPARENT,
            components: Vec::new(),
            default_camera: None,
            default_light: None,
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
    pub fn add_component<T: Any + 'static>(&mut self, component: T) -> usize {
        let b = Box::new(component);
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();
        let raw_ptr = Box::into_raw(b) as *mut dyn Any;
        self.components.push(Component {
            ptr: raw_ptr,
            type_id,
            type_name,
        });
        self.components.len() - 1
    }

    pub fn set_entity_component<T: Any + 'static>(
        &mut self,
        entity_id: usize,
        component: T,
        name: &str,
    ) -> usize {
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

    pub fn get_component<T: Any + 'static>(&self, component_index: usize) -> &mut T {
        let component = self
            .components
            .get(component_index)
            .expect("Invalid component index");

        // 检查类型是否匹配
        // if component.type_id != TypeId::of::<T>() {
        //     panic!(
        //         "Type mismatch: expected {}, but found {}",
        //         std::any::type_name::<T>(),
        //         component.type_name
        //     );
        // }

        let ptr = component.ptr as *mut T;
        assert!(!ptr.is_null(), "Component pointer is null");

        unsafe { &mut *ptr }
    }

    pub fn drop_component<T: Any + 'static>(&mut self, component_index: usize) {
        if component_index >= self.components.len() {
            return;
        }
        let component = &self.components[component_index];
        if component.type_id != TypeId::of::<T>() {
            panic!(
                "Type mismatch while dropping: expected {}, but found {}",
                std::any::type_name::<T>(),
                component.type_name
            );
        }
        let component = self.components.remove(component_index);
        drop(unsafe { Box::from_raw(component.ptr as *mut T) });
    }

    pub fn get_entity_component<T: Any + 'static>(&self, entity: &Entity, component: &str) -> &T {
        let component_ptr = entity.get_component_index(component);
        self.get_component::<T>(component_ptr)
    }

    pub fn get_entity_component_mut<T: Any + 'static>(
        &self,
        entity: &Entity,
        component: &str,
    ) -> &mut T {
        let component_ptr = entity.get_component_index(component);
        self.get_component::<T>(component_ptr)
    }

    pub fn drop_entity_component<T: Any + 'static>(
        &mut self,
        entity: &mut Entity,
        component: &str,
    ) {
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
        let camera_entity: &Entity = self.get_entity(self.default_camera.unwrap());

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
                bind_index: DEFAULT_CAMERA_BIND_INDEX,
            },
            renderer,
        );
        self.set_entity_component::<Box<dyn CameraTrait>>(entity_id, Box::new(camera), "camera");
        self.default_camera = Some(entity_id);
    }

    // need default light for Blinn-Phong shading
    pub fn add_default_directional_light(&mut self, renderer: &renderer::Renderer) {
        let entity_id = self.add_entity(Entity::new());
        let light = DirectionalLight::new(
            &renderer,
            DEFAULT_LIGHT_BIND_INDEX,
            DirectionalLightUniform {
                intensity: 1.,
                direction: [1., 1., -1.],
                color: [1., 1., 0.8, 1.0],
            },
        );
        self.set_entity_component::<Box<dyn LightTrait>>(entity_id, Box::new(light), "light");
        self.default_light = Some(entity_id);
    }
}
