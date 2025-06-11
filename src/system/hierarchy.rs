use crate::{components::transform::Transform, entity::Entity, world::World};

pub struct HierarchySystem;

impl HierarchySystem {
    pub fn new() -> Self {
        Self
    }

    /// 将子实体添加到父实体
    pub fn add_child(&self, world: &mut World, parent: Entity, child: Entity) {
        // 设置子实体的父节点
        if let Some(child_transform) = world.get_component_mut::<Transform>(child) {
            // 如果已经有父节点，先从旧父节点移除
            if let Some(old_parent) = child_transform.get_parent() {
                self.remove_child(world, old_parent, child);
            }

            child_transform.set_parent(Some(parent));
        }

        // 将子实体添加到父实体的子列表
        if let Some(parent_transform) = world.get_component_mut::<Transform>(parent) {
            parent_transform.add_child(child);
        }
    }

    /// 从父实体移除子实体
    pub fn remove_child(&self, world: &mut World, parent: Entity, child: Entity) {
        // 从父实体的子列表中移除
        if let Some(parent_transform) = world.get_component_mut::<Transform>(parent) {
            parent_transform.remove_child(child);
        }

        // 清除子实体的父节点
        if let Some(child_transform) = world.get_component_mut::<Transform>(child) {
            child_transform.set_parent(None);
        }
    }

    /// 获取实体的所有子代（递归）
    pub fn get_descendants(&self, world: &World, entity: Entity) -> Vec<Entity> {
        let mut descendants = Vec::new();
        self.collect_descendants_recursive(world, entity, &mut descendants);
        descendants
    }

    fn collect_descendants_recursive(
        &self,
        world: &World,
        entity: Entity,
        descendants: &mut Vec<Entity>,
    ) {
        if let Some(transform) = world.get_component::<Transform>(entity) {
            for &child in transform.get_children() {
                descendants.push(child);
                self.collect_descendants_recursive(world, child, descendants);
            }
        }
    }

    /// 获取实体到根节点的路径
    pub fn get_path_to_root(&self, world: &World, entity: Entity) -> Vec<Entity> {
        let mut path = vec![entity];
        let mut current = entity;

        while let Some(transform) = world.get_component::<Transform>(current) {
            if let Some(parent) = transform.get_parent() {
                path.push(parent);
                current = parent;
            } else {
                break;
            }
        }

        path.reverse();
        path
    }
}
