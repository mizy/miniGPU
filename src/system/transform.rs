use crate::{components::transform::Transform, entity::Entity, world::World};

pub struct TransformSystem;

impl TransformSystem {
    pub fn new() -> Self {
        Self
    }

    /// 更新所有 Transform 组件的矩阵
    pub fn update(&mut self, world: &mut World) {
        // 第一步：更新所有本地矩阵
        self.update_local_matrices(world);

        // 第二步：更新世界矩阵（层级顺序）
        self.update_world_matrices(world);
    }

    fn update_local_matrices(&self, world: &mut World) {
        // 获取所有需要更新的 Transform
        let entities_to_update: Vec<Entity> = world
            .query::<Transform>()
            .filter(|(_, transform)| transform.is_local_dirty())
            .map(|(entity, _)| entity)
            .collect();

        // 更新本地矩阵
        for entity in entities_to_update {
            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                transform.update_local_matrix();
            }
        }
    }

    fn update_world_matrices(&self, world: &mut World) {
        // 收集所有根节点（没有父节点的 Transform）
        let root_entities: Vec<Entity> = world
            .query::<Transform>()
            .filter(|(_, transform)| !transform.has_parent())
            .map(|(entity, _)| entity)
            .collect();

        // 递归更新每个根节点及其子节点
        for root_entity in root_entities {
            self.update_hierarchy_recursive(world, root_entity, None);
        }
    }

    fn update_hierarchy_recursive(
        &self,
        world: &mut World,
        entity: Entity,
        parent_world_matrix: Option<glam::Mat4>,
    ) {
        // 更新当前实体的世界矩阵
        let (current_world_matrix, children) = {
            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                transform.update_world_matrix(parent_world_matrix);
                let world_matrix = transform.get_world_matrix();
                let children = transform.get_children().to_vec();
                (world_matrix, children)
            } else {
                return;
            }
        };

        // 递归更新所有子节点
        for child in children {
            self.update_hierarchy_recursive(world, child, Some(current_world_matrix));
        }
    }
}
