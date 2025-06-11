use glam::{Mat4, Quat, Vec3};

/// 实例数据 - 每个实例的变换信息
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    /// 模型矩阵 (4x4 矩阵分成 4 个 Vec4)
    pub model_matrix: [[f32; 4]; 4],
    /// 法线矩阵 (3x3 矩阵，用于变换法线，这里用 4x4 存储以满足对齐要求)
    pub normal_matrix: [[f32; 4]; 4],
    /// 实例 ID (可用于 shader 中的特殊处理)
    pub instance_id: u32,
    /// 填充字节以满足对齐要求
    pub _padding: [u32; 3],
}

impl InstanceData {
    /// 从变换矩阵创建实例数据
    pub fn new(transform: Mat4, instance_id: u32) -> Self {
        let normal_matrix = transform.inverse().transpose();

        Self {
            model_matrix: transform.to_cols_array_2d(),
            normal_matrix: normal_matrix.to_cols_array_2d(),
            instance_id,
            _padding: [0; 3],
        }
    }

    /// 从位置、旋转、缩放创建实例数据
    pub fn from_transform(position: Vec3, rotation: Quat, scale: Vec3, instance_id: u32) -> Self {
        let transform = Mat4::from_scale_rotation_translation(scale, rotation, position);
        Self::new(transform, instance_id)
    }

    /// 简单的位置变换
    pub fn from_position(position: Vec3, instance_id: u32) -> Self {
        let transform = Mat4::from_translation(position);
        Self::new(transform, instance_id)
    }
}

impl Default for InstanceData {
    fn default() -> Self {
        Self::new(Mat4::IDENTITY, 0)
    }
}

/// Instance 组件 - 纯数据结构，只存储实例数据
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    /// 实例数据数组
    pub data: Vec<InstanceData>,
    /// 标记数据是否已更改（用于系统判断是否需要更新 GPU 缓冲区）
    pub dirty: bool,
}

impl Instance {
    /// 创建新的 Instance 组件
    pub fn new(data: Vec<InstanceData>) -> Self {
        Self {
            data,
            dirty: true, // 新创建的实例标记为脏
        }
    }

    /// 创建单个实例
    pub fn single(instance_data: InstanceData) -> Self {
        Self::new(vec![instance_data])
    }

    /// 从变换矩阵数组创建
    pub fn from_transforms(transforms: Vec<Mat4>) -> Self {
        let data: Vec<InstanceData> = transforms
            .into_iter()
            .enumerate()
            .map(|(i, transform)| InstanceData::new(transform, i as u32))
            .collect();
        Self::new(data)
    }

    /// 从位置数组创建 (简单的平移实例)
    pub fn from_positions(positions: Vec<Vec3>) -> Self {
        let data: Vec<InstanceData> = positions
            .into_iter()
            .enumerate()
            .map(|(i, pos)| InstanceData::from_position(pos, i as u32))
            .collect();
        Self::new(data)
    }

    /// 添加实例
    pub fn add_instance(&mut self, instance_data: InstanceData) {
        self.data.push(instance_data);
        self.dirty = true;
    }

    /// 更新指定索引的实例
    pub fn update_instance(&mut self, index: usize, instance_data: InstanceData) {
        if index < self.data.len() {
            self.data[index] = instance_data;
            self.dirty = true;
        }
    }

    /// 移除实例
    pub fn remove_instance(&mut self, index: usize) {
        if index < self.data.len() {
            self.data.remove(index);
            self.dirty = true;
        }
    }

    /// 清空所有实例
    pub fn clear(&mut self) {
        self.data.clear();
        self.dirty = true;
    }

    /// 获取实例数量
    pub fn count(&self) -> usize {
        self.data.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 标记为需要更新
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// 标记为已更新
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// 检查是否需要更新
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
