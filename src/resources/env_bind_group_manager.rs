use std::collections::HashMap;
use wgpu::{BindGroup, BindGroupLayout};

/// 环境绑定组 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnvBindGroupId(pub u32);

impl From<u32> for EnvBindGroupId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl EnvBindGroupId {
    pub fn invalid() -> Self {
        Self(u32::MAX)
    }

    pub fn is_valid(&self) -> bool {
        self.0 != u32::MAX
    }
}

/// 环境绑定组资源
pub struct EnvBindGroup {
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
    pub index: u32,
    pub label: Option<String>,
}

/// 环境绑定组管理器 - 只提供 CRUD 功能
pub struct EnvBindGroupManager {
    bind_groups: HashMap<EnvBindGroupId, EnvBindGroup>,
    bind_groups_by_name: HashMap<String, EnvBindGroupId>,
    next_id: u32,
}

impl EnvBindGroupManager {
    pub fn new() -> Self {
        Self {
            bind_groups: HashMap::new(),
            bind_groups_by_name: HashMap::new(),
            next_id: 1,
        }
    }

    /// 添加绑定组
    pub fn add_bind_group(
        &mut self,
        bind_group: BindGroup,
        bind_group_layout: BindGroupLayout,
        index: u32,
        label: Option<String>,
    ) -> EnvBindGroupId {
        let id = EnvBindGroupId(self.next_id);
        self.next_id += 1;

        let env_bind_group = EnvBindGroup {
            bind_group,
            bind_group_layout,
            index,
            label: label.clone(),
        };

        self.bind_groups.insert(id, env_bind_group);

        // 添加名称映射
        if let Some(name) = label {
            self.bind_groups_by_name.insert(name, id);
        }

        id
    }

    /// 获取绑定组
    pub fn get_bind_group(&self, id: EnvBindGroupId) -> Option<&EnvBindGroup> {
        self.bind_groups.get(&id)
    }

    /// 获取可变绑定组
    pub fn get_bind_group_mut(&mut self, id: EnvBindGroupId) -> Option<&mut EnvBindGroup> {
        self.bind_groups.get_mut(&id)
    }

    /// 通过名称获取绑定组
    pub fn get_bind_group_by_name(&self, name: &str) -> Option<&EnvBindGroup> {
        self.bind_groups_by_name
            .get(name)
            .and_then(|id| self.bind_groups.get(id))
    }

    /// 通过名称获取绑定组 ID
    pub fn get_bind_group_id_by_name(&self, name: &str) -> Option<EnvBindGroupId> {
        self.bind_groups_by_name.get(name).copied()
    }

    /// 更新绑定组
    pub fn update_bind_group(
        &mut self,
        id: EnvBindGroupId,
        bind_group: BindGroup,
        bind_group_layout: BindGroupLayout,
        index: u32,
    ) -> Result<(), String> {
        let env_bind_group = self
            .bind_groups
            .get_mut(&id)
            .ok_or("Bind group not found")?;

        env_bind_group.bind_group = bind_group;
        env_bind_group.bind_group_layout = bind_group_layout;
        env_bind_group.index = index;

        Ok(())
    }

    /// 删除绑定组
    pub fn remove_bind_group(&mut self, id: EnvBindGroupId) -> Option<EnvBindGroup> {
        if let Some(env_bind_group) = self.bind_groups.remove(&id) {
            // 从名称映射中移除
            if let Some(ref label) = env_bind_group.label {
                self.bind_groups_by_name.remove(label);
            }
            Some(env_bind_group)
        } else {
            None
        }
    }

    /// 删除指定名称的绑定组
    pub fn remove_bind_group_by_name(&mut self, name: &str) -> Option<EnvBindGroup> {
        if let Some(id) = self.bind_groups_by_name.remove(name) {
            self.bind_groups.remove(&id)
        } else {
            None
        }
    }

    /// 获取所有绑定组
    pub fn get_all_bind_groups(&self) -> Vec<&EnvBindGroup> {
        self.bind_groups.values().collect()
    }

    /// 获取所有绑定组布局
    pub fn get_all_bind_group_layouts(&self) -> Vec<&BindGroupLayout> {
        self.bind_groups
            .values()
            .map(|bg| &bg.bind_group_layout)
            .collect()
    }

    /// 检查绑定组是否存在
    pub fn contains(&self, id: EnvBindGroupId) -> bool {
        self.bind_groups.contains_key(&id)
    }

    /// 检查名称是否存在
    pub fn contains_name(&self, name: &str) -> bool {
        self.bind_groups_by_name.contains_key(name)
    }

    /// 获取绑定组数量
    pub fn count(&self) -> usize {
        self.bind_groups.len()
    }

    /// 清空所有绑定组
    pub fn clear(&mut self) {
        self.bind_groups.clear();
        self.bind_groups_by_name.clear();
    }

    /// 列出所有绑定组名称
    pub fn list_names(&self) -> Vec<&String> {
        self.bind_groups_by_name.keys().collect()
    }
}

impl Default for EnvBindGroupManager {
    fn default() -> Self {
        Self::new()
    }
}
