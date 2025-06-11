use std::collections::HashMap;
use wgpu::{util::DeviceExt, VertexBufferLayout, VertexStepMode};

use crate::{
    components::instance::InstanceData,
    entity::Entity,
    renderer::Renderer,
    resources::{env_bind_group_manager::EnvBindGroupManager, material::MaterialTrait, mesh::Mesh},
};

// === ID 类型定义 ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferId(pub u32);

// === 缓冲区类型枚举 ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferType {
    Transform,
    Light,
    Camera,
    Material,
    Instance,
    Custom(u32), // 自定义类型，用数字标识
}

// === 缓冲区用途枚举 ===
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferUsage {
    Uniform,
    Storage,
    Vertex,
    Index,
}

// === 通用缓冲区结构 ===
#[derive(Debug)]
pub struct BufferResource {
    pub buffer: wgpu::Buffer,
    pub buffer_type: BufferType,
    pub usage: BufferUsage,
    pub size: usize,
    pub bind_index: Option<u32>, // 在 shader 中的绑定索引（如果适用）
    pub label: Option<String>,
}

impl BufferResource {
    pub fn new(
        buffer: wgpu::Buffer,
        buffer_type: BufferType,
        usage: BufferUsage,
        size: usize,
        bind_index: Option<u32>,
        label: Option<String>,
    ) -> Self {
        Self {
            buffer,
            buffer_type,
            usage,
            size,
            bind_index,
            label,
        }
    }
}

// === ID 实现 ===
macro_rules! impl_id {
    ($id_type:ty) => {
        impl Default for $id_type {
            fn default() -> Self {
                Self(u32::MAX)
            }
        }

        impl From<u32> for $id_type {
            fn from(id: u32) -> Self {
                Self(id)
            }
        }

        impl $id_type {
            pub fn invalid() -> Self {
                Self(u32::MAX)
            }

            pub fn is_valid(&self) -> bool {
                self.0 != u32::MAX
            }
        }
    };
}

impl_id!(MeshId);
impl_id!(MaterialId);
impl_id!(BufferId);
// 添加 BufferLayoutId 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferLayoutId(pub u32);

impl_id!(BufferLayoutId);

// 缓冲区布局资源
#[derive(Debug, Clone)]
pub struct BufferLayoutResource {
    pub layout: VertexBufferLayout<'static>,
    pub buffer_type: BufferType,
    pub step_mode: VertexStepMode,
    pub label: Option<String>,
}

impl BufferLayoutResource {
    pub fn new(
        layout: VertexBufferLayout<'static>,
        buffer_type: BufferType,
        step_mode: VertexStepMode,
        label: Option<String>,
    ) -> Self {
        Self {
            layout,
            buffer_type,
            step_mode,
            label,
        }
    }
}

// === 缓冲区创建参数 ===
#[derive(Debug)]
pub struct BufferCreateInfo<'a> {
    pub buffer_type: BufferType,
    pub usage: BufferUsage,
    pub data: &'a [u8],
    pub bind_index: Option<u32>,
    pub label: Option<String>,
}

impl<'a> BufferCreateInfo<'a> {
    pub fn new(buffer_type: BufferType, usage: BufferUsage, data: &'a [u8]) -> Self {
        Self {
            buffer_type,
            usage,
            data,
            bind_index: None,
            label: None,
        }
    }

    pub fn with_bind_index(mut self, bind_index: u32) -> Self {
        self.bind_index = Some(bind_index);
        self
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
}

// === 主要资源管理器 ===
pub struct ResourceManager {
    // 现有资源存储
    meshes: HashMap<MeshId, Mesh>,
    materials: HashMap<MaterialId, Box<dyn MaterialTrait>>,

    // 统一缓冲区管理
    buffers: HashMap<BufferId, BufferResource>,

    instance_buffers: HashMap<Entity, BufferId>,

    // 缓冲区布局管理
    buffer_layouts: HashMap<BufferLayoutId, BufferLayoutResource>,
    buffer_layout_names: HashMap<String, BufferLayoutId>,

    next_buffer_layout_id: u32,

    // ID 生成器
    next_mesh_id: u32,
    next_material_id: u32,
    next_buffer_id: u32,

    // 名称映射
    mesh_names: HashMap<String, MeshId>,
    material_names: HashMap<String, MaterialId>,
    buffer_names: HashMap<String, BufferId>,

    // 类型化缓冲区索引（便于查找特定类型的缓冲区）
    buffers_by_type: HashMap<BufferType, Vec<BufferId>>,
    env_bind_group_manager: EnvBindGroupManager,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            materials: HashMap::new(),
            buffers: HashMap::new(),
            next_mesh_id: 1,
            next_material_id: 1,
            next_buffer_id: 1,
            mesh_names: HashMap::new(),
            material_names: HashMap::new(),
            buffer_names: HashMap::new(),
            buffers_by_type: HashMap::new(),
            env_bind_group_manager: EnvBindGroupManager::new(),
            buffer_layouts: HashMap::new(),
            next_buffer_layout_id: 1,
            instance_buffers: HashMap::new(),
            buffer_layout_names: HashMap::new(),
        }
    }

    // === 缓冲区管理方法 ===

    /// 创建缓冲区
    pub fn create_buffer(
        &mut self,
        renderer: &Renderer,
        create_info: BufferCreateInfo,
    ) -> BufferId {
        let device = &renderer.device;

        // 将 BufferUsage 转换为 wgpu::BufferUsages
        let wgpu_usage = match create_info.usage {
            BufferUsage::Uniform => wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            BufferUsage::Storage => wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            BufferUsage::Vertex => wgpu::BufferUsages::VERTEX,
            BufferUsage::Index => wgpu::BufferUsages::INDEX,
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: create_info.label.as_deref(),
            contents: create_info.data,
            usage: wgpu_usage,
        });

        let buffer_resource = BufferResource::new(
            buffer,
            create_info.buffer_type,
            create_info.usage,
            create_info.data.len(),
            create_info.bind_index,
            create_info.label.clone(),
        );

        let id = BufferId(self.next_buffer_id);
        self.next_buffer_id += 1;

        // 添加到类型索引
        self.buffers_by_type
            .entry(create_info.buffer_type)
            .or_insert_with(Vec::new)
            .push(id);

        // 添加名称映射
        if let Some(label) = create_info.label {
            self.buffer_names.insert(label, id);
        }

        self.buffers.insert(id, buffer_resource);
        id
    }

    /// 获取缓冲区
    pub fn get_buffer(&self, id: BufferId) -> Option<&BufferResource> {
        self.buffers.get(&id)
    }

    /// 获取可变缓冲区
    pub fn get_buffer_mut(&mut self, id: BufferId) -> Option<&mut BufferResource> {
        self.buffers.get_mut(&id)
    }

    /// 通过名称获取缓冲区
    pub fn get_buffer_by_name(&self, name: &str) -> Option<&BufferResource> {
        self.buffer_names
            .get(name)
            .and_then(|id| self.buffers.get(id))
    }

    /// 更新缓冲区数据
    pub fn update_buffer(
        &self,
        renderer: &Renderer,
        id: BufferId,
        data: &[u8],
        offset: u64,
    ) -> Result<(), String> {
        let buffer_resource = self.buffers.get(&id).ok_or("Buffer not found")?;

        if offset as usize + data.len() > buffer_resource.size {
            return Err("Data exceeds buffer size".to_string());
        }

        renderer
            .queue
            .write_buffer(&buffer_resource.buffer, offset, data);
        Ok(())
    }

    /// 获取指定类型的所有缓冲区
    pub fn get_buffers_by_type(&self, buffer_type: BufferType) -> Vec<&BufferResource> {
        self.buffers_by_type
            .get(&buffer_type)
            .map(|ids| ids.iter().filter_map(|id| self.buffers.get(id)).collect())
            .unwrap_or_default()
    }

    /// 删除缓冲区
    pub fn remove_buffer(&mut self, id: BufferId) -> Option<BufferResource> {
        if let Some(buffer_resource) = self.buffers.remove(&id) {
            // 从类型索引中移除
            if let Some(type_buffers) = self.buffers_by_type.get_mut(&buffer_resource.buffer_type) {
                type_buffers.retain(|&buffer_id| buffer_id != id);
            }

            // 从名称映射中移除
            self.buffer_names
                .retain(|_, &mut buffer_id| buffer_id != id);

            Some(buffer_resource)
        } else {
            None
        }
    }
    // === 现有的网格和材质方法保持不变 ===

    pub fn add_mesh(&mut self, mesh: Mesh, name: Option<String>) -> MeshId {
        let id = MeshId(self.next_mesh_id);
        self.next_mesh_id += 1;
        self.meshes.insert(id, mesh);

        if let Some(name) = name {
            self.mesh_names.insert(name, id);
        }

        id
    }

    pub fn get_mesh(&self, id: MeshId) -> Option<&Mesh> {
        self.meshes.get(&id)
    }

    pub fn get_mesh_mut(&mut self, id: MeshId) -> Option<&mut Mesh> {
        self.meshes.get_mut(&id)
    }

    pub fn add_material(
        &mut self,
        material: Box<dyn MaterialTrait>,
        name: Option<String>,
    ) -> MaterialId {
        let id = MaterialId(self.next_material_id);
        self.next_material_id += 1;
        self.materials.insert(id, material);

        if let Some(name) = name {
            self.material_names.insert(name, id);
        }
        id
    }

    pub fn get_material(&self, id: MaterialId) -> Option<&Box<dyn MaterialTrait>> {
        self.materials.get(&id)
    }

    pub fn get_material_mut(&mut self, id: MaterialId) -> Option<&mut Box<dyn MaterialTrait>> {
        self.materials.get_mut(&id)
    }

    // === 资源统计 ===

    /// 获取缓冲区数量
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }

    /// 获取指定类型缓冲区数量
    pub fn buffer_count_by_type(&self, buffer_type: BufferType) -> usize {
        self.buffers_by_type
            .get(&buffer_type)
            .map(|buffers| buffers.len())
            .unwrap_or(0)
    }

    /// 获取缓冲区总内存使用量
    pub fn total_buffer_memory(&self) -> usize {
        self.buffers.values().map(|buffer| buffer.size).sum()
    }

    /// 注册缓冲区布局
    pub fn register_buffer_layout(
        &mut self,
        layout: VertexBufferLayout<'static>,
        buffer_type: BufferType,
        step_mode: VertexStepMode,
        name: Option<String>,
    ) -> BufferLayoutId {
        let layout_resource =
            BufferLayoutResource::new(layout, buffer_type, step_mode, name.clone());

        let id = BufferLayoutId(self.next_buffer_layout_id);
        self.next_buffer_layout_id += 1;

        if let Some(name) = name {
            self.buffer_layout_names.insert(name, id);
        }

        self.buffer_layouts.insert(id, layout_resource);
        id
    }

    pub fn sync_instance_buffer(
        &mut self,
        entity: Entity,
        instance_data: &[InstanceData],
        renderer: &Renderer,
    ) -> Option<BufferId> {
        // 如果实例数据为空，移除缓冲区
        if instance_data.is_empty() {
            return self.remove_instance_buffer(entity);
        }

        let create_info = BufferCreateInfo::new(
            BufferType::Instance,
            BufferUsage::Vertex,
            bytemuck::cast_slice(instance_data),
        )
        .with_label(format!("Instance Buffer Entity {:?}", entity));

        // 检查是否已存在缓冲区
        if let Some(&existing_buffer_id) = self.instance_buffers.get(&entity) {
            // 检查现有缓冲区
            if let Some(existing_buffer) = self.get_buffer(existing_buffer_id) {
                let new_size = instance_data.len() * std::mem::size_of::<InstanceData>();

                if new_size != existing_buffer.size {
                    // 大小变化，需要重新创建缓冲区
                    self.remove_buffer(existing_buffer_id);
                    let new_buffer_id = self.create_buffer(renderer, create_info);
                    self.instance_buffers.insert(entity, new_buffer_id);
                    Some(new_buffer_id)
                } else {
                    // 只更新数据
                    match self.update_buffer(
                        renderer,
                        existing_buffer_id,
                        bytemuck::cast_slice(instance_data),
                        0,
                    ) {
                        Ok(_) => Some(existing_buffer_id),
                        Err(e) => {
                            eprintln!("Failed to update instance buffer: {}", e);
                            // 尝试重新创建
                            self.remove_buffer(existing_buffer_id);
                            let new_buffer_id = self.create_buffer(renderer, create_info);
                            self.instance_buffers.insert(entity, new_buffer_id);
                            Some(new_buffer_id)
                        }
                    }
                }
            } else {
                // 缓冲区不存在，创建新的
                let new_buffer_id = self.create_buffer(renderer, create_info);
                self.instance_buffers.insert(entity, new_buffer_id);
                Some(new_buffer_id)
            }
        } else {
            // 创建新缓冲区
            let buffer_id = self.create_buffer(renderer, create_info);
            self.instance_buffers.insert(entity, buffer_id);
            Some(buffer_id)
        }
    }

    /// 获取实体的实例缓冲区 ID
    pub fn get_instance_buffer_id(&self, entity: Entity) -> Option<BufferId> {
        self.instance_buffers.get(&entity).copied()
    }

    /// 移除实体的实例缓冲区
    pub fn remove_instance_buffer(&mut self, entity: Entity) -> Option<BufferId> {
        if let Some(buffer_id) = self.instance_buffers.remove(&entity) {
            self.remove_buffer(buffer_id);
            Some(buffer_id)
        } else {
            None
        }
    }

    pub fn get_buffer_layout_by_name(&self, name: &str) -> Option<&BufferLayoutResource> {
        self.buffer_layout_names
            .get(name)
            .and_then(|id| self.buffer_layouts.get(id))
    }

    /// 初始化标准布局
    fn init_standard_layouts(&mut self) {
        self.create_instance_buffer_layout();
        // 可以添加其他标准布局...
    }

    /// 创建标准的实例缓冲区布局
    fn create_instance_buffer_layout(&mut self) -> BufferLayoutId {
        use wgpu::{VertexAttribute, VertexFormat, VertexStepMode};

        let layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                // 模型矩阵 (4x4) - 需要 4 个 location
                VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
                // 法线矩阵 (4x4) - 需要 4 个 location
                VertexAttribute {
                    offset: std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: (std::mem::size_of::<[[f32; 4]; 4]>() + std::mem::size_of::<[f32; 4]>())
                        as wgpu::BufferAddress,
                    shader_location: 10,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: (std::mem::size_of::<[[f32; 4]; 4]>() + std::mem::size_of::<[f32; 8]>())
                        as wgpu::BufferAddress,
                    shader_location: 11,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: (std::mem::size_of::<[[f32; 4]; 4]>()
                        + std::mem::size_of::<[f32; 12]>())
                        as wgpu::BufferAddress,
                    shader_location: 12,
                    format: VertexFormat::Float32x4,
                },
                // 实例 ID
                VertexAttribute {
                    offset: (std::mem::size_of::<[[f32; 4]; 4]>() * 2) as wgpu::BufferAddress,
                    shader_location: 13,
                    format: VertexFormat::Uint32,
                },
            ],
        };

        self.register_buffer_layout(
            layout,
            BufferType::Instance,
            VertexStepMode::Instance,
            Some("instance".to_string()),
        )
    }

    /// 获取缓冲区布局
    pub fn get_buffer_layout(&self, id: BufferLayoutId) -> Option<&BufferLayoutResource> {
        self.buffer_layouts.get(&id)
    }

    /// 清理所有资源
    pub fn cleanup(&mut self) {
        self.meshes.clear();
        self.materials.clear();
        self.buffers.clear();
        self.buffer_layouts.clear(); // 添加
        self.instance_buffers.clear(); // 添加
        self.buffer_layout_names.clear(); // 添加

        self.mesh_names.clear();
        self.material_names.clear();
        self.buffer_names.clear();
        self.buffers_by_type.clear();

        self.next_mesh_id = 1;
        self.next_material_id = 1;
        self.next_buffer_id = 1;
        self.next_buffer_layout_id = 1; // 添加
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}
