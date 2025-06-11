use glam::{Mat4, Quat, Vec3};

use crate::resources::resource_manager::BufferId;

#[derive(Debug, Clone)]
pub struct Transform {
    // 基础变换属性
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,

    // 缓存的矩阵
    local_matrix: Mat4,
    world_matrix: Mat4,

    // 脏标记
    local_dirty: bool,
    world_dirty: bool,

    // 层级关系
    pub parent: Option<crate::entity::Entity>,
    pub children: Vec<crate::entity::Entity>,

    pub buffer_id: Option<BufferId>,
    pub buffer_dirty: bool, // 标记缓冲区是否需要更新
}

impl Transform {
    /// 创建新的 Transform
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let local_matrix = Mat4::from_scale_rotation_translation(scale, rotation, position);
        Self {
            position,
            rotation,
            scale,
            local_matrix,
            world_matrix: local_matrix,
            local_dirty: false,
            world_dirty: false,
            parent: None,
            children: Vec::new(),
            buffer_id: None,    // 初始时没有缓冲区
            buffer_dirty: true, // 需要创建缓冲区
        }
    }

    /// 创建身份变换
    pub fn identity() -> Self {
        Self::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE)
    }

    /// 从位置创建
    pub fn from_translation(position: Vec3) -> Self {
        Self::new(position, Quat::IDENTITY, Vec3::ONE)
    }

    /// 从旋转创建
    pub fn from_rotation(rotation: Quat) -> Self {
        Self::new(Vec3::ZERO, rotation, Vec3::ONE)
    }

    /// 从缩放创建
    pub fn from_scale(scale: Vec3) -> Self {
        Self::new(Vec3::ZERO, Quat::IDENTITY, scale)
    }

    // === 位置操作 ===
    pub fn set_position(&mut self, position: Vec3) {
        if self.position != position {
            self.position = position;
            self.mark_local_dirty();
        }
    }

    pub fn translate(&mut self, delta: Vec3) {
        self.set_position(self.position + delta);
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    // === 旋转操作 ===
    pub fn set_rotation(&mut self, rotation: Quat) {
        if self.rotation != rotation {
            self.rotation = rotation;
            self.mark_local_dirty();
        }
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.set_rotation(rotation * self.rotation);
    }

    pub fn rotate_around_axis(&mut self, axis: Vec3, angle: f32) {
        let rotation = Quat::from_axis_angle(axis.normalize(), angle);
        self.rotate(rotation);
    }

    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.position).normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward);

        // 构建旋转矩阵并转换为四元数
        let rotation_matrix = Mat4::from_cols(
            right.extend(0.0),
            up.extend(0.0),
            (-forward).extend(0.0),
            Vec3::ZERO.extend(1.0),
        );

        self.set_rotation(Quat::from_mat4(&rotation_matrix));
    }

    pub fn get_rotation(&self) -> Quat {
        self.rotation
    }

    // === 缩放操作 ===
    pub fn set_scale(&mut self, scale: Vec3) {
        if self.scale != scale {
            self.scale = scale;
            self.mark_local_dirty();
        }
    }

    pub fn set_uniform_scale(&mut self, scale: f32) {
        self.set_scale(Vec3::splat(scale));
    }

    pub fn get_scale(&self) -> Vec3 {
        self.scale
    }

    // === 矩阵操作 ===
    pub fn get_local_matrix(&self) -> Mat4 {
        if self.local_dirty {
            // 重新计算但不修改自身（因为是 &self）
            Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
        } else {
            self.local_matrix
        }
    }

    pub fn get_world_matrix(&self) -> Mat4 {
        self.world_matrix
    }

    /// 更新本地矩阵（由系统调用）
    pub fn update_local_matrix(&mut self) {
        if self.local_dirty {
            self.local_matrix =
                Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
            self.local_dirty = false;
            self.world_dirty = true;
        }
    }

    /// 更新世界矩阵（由系统调用）
    pub fn update_world_matrix(&mut self, parent_world_matrix: Option<Mat4>) {
        if self.world_dirty || parent_world_matrix.is_some() {
            self.world_matrix = if let Some(parent_matrix) = parent_world_matrix {
                parent_matrix * self.local_matrix
            } else {
                self.local_matrix
            };
            self.world_dirty = false;
        }
    }

    // === 方向向量 ===
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    // === 状态查询 ===
    pub fn is_local_dirty(&self) -> bool {
        self.local_dirty
    }

    pub fn is_world_dirty(&self) -> bool {
        self.world_dirty
    }

    pub fn is_dirty(&self) -> bool {
        self.local_dirty || self.world_dirty
    }

    // === 层级操作 ===
    pub fn add_child(&mut self, child: crate::entity::Entity) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    pub fn remove_child(&mut self, child: crate::entity::Entity) {
        self.children.retain(|&c| c != child);
    }

    pub fn set_parent(&mut self, parent: Option<crate::entity::Entity>) {
        if self.parent != parent {
            self.parent = parent;
            self.world_dirty = true;
        }
    }

    pub fn get_children(&self) -> &[crate::entity::Entity] {
        &self.children
    }

    pub fn get_parent(&self) -> Option<crate::entity::Entity> {
        self.parent
    }

    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// 标记缓冲区需要更新
    pub fn mark_buffer_dirty(&mut self) {
        self.buffer_dirty = true;
    }

    /// 检查缓冲区是否需要更新
    pub fn is_buffer_dirty(&self) -> bool {
        self.buffer_dirty
    }

    /// 清除缓冲区脏标记
    pub fn clear_buffer_dirty(&mut self) {
        self.buffer_dirty = false;
    }

    /// 设置缓冲区 ID
    pub fn set_buffer_id(&mut self, buffer_id: BufferId) {
        self.buffer_id = Some(buffer_id);
    }

    /// 获取缓冲区 ID
    pub fn get_buffer_id(&self) -> Option<BufferId> {
        self.buffer_id
    }

    /// 检查是否有有效的缓冲区
    pub fn has_valid_buffer(&self) -> bool {
        self.buffer_id.map(|id| id.is_valid()).unwrap_or(false)
    }

    // 重写内部的脏标记方法
    fn mark_local_dirty(&mut self) {
        self.local_dirty = true;
        self.world_dirty = true;
        self.buffer_dirty = true; // 同时标记缓冲区需要更新
    }

    pub fn mark_world_dirty(&mut self) {
        self.world_dirty = true;
        self.buffer_dirty = true; // 同时标记缓冲区需要更新
    }

    // === 组合变换 ===
    pub fn apply_transform(&mut self, other: &Transform) {
        self.set_position(self.position + other.position);
        self.set_rotation(self.rotation * other.rotation);
        self.set_scale(self.scale * other.scale);
    }

    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform::new(
            self.position.lerp(other.position, t),
            self.rotation.slerp(other.rotation, t),
            self.scale.lerp(other.scale, t),
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

// === 构建器模式 ===
pub struct TransformBuilder {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl TransformBuilder {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn translation(self, x: f32, y: f32, z: f32) -> Self {
        self.position(Vec3::new(x, y, z))
    }

    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn rotation_euler(self, x: f32, y: f32, z: f32) -> Self {
        self.rotation(Quat::from_euler(glam::EulerRot::XYZ, x, y, z))
    }

    pub fn rotation_axis_angle(self, axis: Vec3, angle: f32) -> Self {
        self.rotation(Quat::from_axis_angle(axis.normalize(), angle))
    }

    pub fn scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn uniform_scale(self, scale: f32) -> Self {
        self.scale(Vec3::splat(scale))
    }

    pub fn scale_xyz(self, x: f32, y: f32, z: f32) -> Self {
        self.scale(Vec3::new(x, y, z))
    }

    pub fn build(self) -> Transform {
        Transform::new(self.position, self.rotation, self.scale)
    }
}

impl Default for TransformBuilder {
    fn default() -> Self {
        Self::new()
    }
}
