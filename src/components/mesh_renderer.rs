use crate::resources::resource_manager::{MaterialId, MeshId};

/// 网格渲染组件 - ECS 系统中负责渲染的核心组件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshRenderer {
    /// 网格资源 ID
    pub mesh_id: MeshId,
    /// 材质资源 ID  
    pub material_id: MaterialId,
    /// 是否可见/启用渲染
    pub visible: bool,
    /// 渲染层级（用于排序，数值越大越后渲染）
    pub layer: i32,
    /// 是否投射阴影
    pub cast_shadows: bool,
    /// 是否接收阴影
    pub receive_shadows: bool,
    /// 渲染优先级（同层级内的排序）
    pub priority: i32,
}

impl MeshRenderer {
    /// 创建新的网格渲染组件
    pub fn new(mesh_id: MeshId, material_id: MaterialId) -> Self {
        Self {
            mesh_id,
            material_id,
            visible: true,
            layer: 0,
            cast_shadows: true,
            receive_shadows: true,
            priority: 0,
        }
    }

    /// 创建不可见的渲染组件
    pub fn invisible(mesh_id: MeshId, material_id: MaterialId) -> Self {
        Self {
            mesh_id,
            material_id,
            visible: false,
            layer: 0,
            cast_shadows: false,
            receive_shadows: false,
            priority: 0,
        }
    }

    /// 流畅 API：设置可见性
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// 流畅 API：设置渲染层级
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    /// 流畅 API：设置阴影选项
    pub fn with_shadows(mut self, cast: bool, receive: bool) -> Self {
        self.cast_shadows = cast;
        self.receive_shadows = receive;
        self
    }

    /// 流畅 API：设置渲染优先级
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// 显示渲染组件
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// 隐藏渲染组件
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// 切换可见性
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// 检查是否应该渲染（可见且资源有效）
    pub fn should_render(&self) -> bool {
        self.visible && self.is_valid()
    }

    /// 检查资源 ID 是否有效
    pub fn is_valid(&self) -> bool {
        self.mesh_id.is_valid() && self.material_id.is_valid()
    }

    /// 更新网格资源
    pub fn set_mesh(&mut self, mesh_id: MeshId) {
        self.mesh_id = mesh_id;
    }

    /// 更新材质资源
    pub fn set_material(&mut self, material_id: MaterialId) {
        self.material_id = material_id;
    }

    /// 获取渲染排序键（用于渲染系统排序）
    pub fn sort_key(&self) -> (i32, i32) {
        (self.layer, self.priority)
    }
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self {
            mesh_id: MeshId::invalid(),
            material_id: MaterialId::invalid(),
            visible: false,
            layer: 0,
            cast_shadows: false,
            receive_shadows: false,
            priority: 0,
        }
    }
}
