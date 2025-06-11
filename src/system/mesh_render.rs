use std::collections::HashMap;

use gltf::Mesh;
use wgpu::{util::DeviceExt, CommandEncoder, StoreOp, VertexBufferLayout};

use crate::{
    components::{
        camera::{
            camera_common::{CameraGpuData, CameraUniform},
            orthographic::OrthographicCamera,
            perspective::PerspectiveCamera,
        },
        instance::Instance,
        lights::{
            ambient_light::{AmbientLight, AmbientLightUniform},
            directional_light::{DirectionalLight, DirectionalLightUniform},
        },
        mesh_renderer::MeshRenderer,
    },
    entity::Entity,
    renderer::Renderer,
    resources::{
        material::MaterialTrait,
        resource_manager::{self, MaterialId},
    },
    world::{self, World},
};

use super::system::System;

pub struct MeshRender {
    pipeline_cache: HashMap<PipelineKey, wgpu::RenderPipeline>,
}

pub struct EnvBindGroup {
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub index: u32,
}

pub struct RenderOptions<'a> {
    entities: Vec<Entity>,
    renderer: &'a Renderer,
    world: &'a mut World,
    render_pass: wgpu::RenderPass<'a>,
    env_pipeline_layouts: &'a Vec<&'a wgpu::BindGroupLayout>,
    env_bind_groups: &'a Vec<EnvBindGroup>,
}

impl System for MeshRender {
    fn update(&mut self, world: &mut World, delta_time: f32) {}
    fn render(&mut self, scene: &mut World, renderer: &Renderer, delta_time: f32) {
        let frame = renderer
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.render(&mut encoder, &view, scene, renderer);
        renderer.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
const ENV_BIND_GROUP_INDEX: u32 = 1;
const LIGHT_BINDING_INDEX: u32 = 2;

pub struct LightCollection {
    pub ambient_lights: Vec<Entity>,
    pub directional_lights: Vec<Entity>,
}
// 光源数量限制
const MAX_DIRECTIONAL_LIGHTS: usize = 8;
const MAX_AMBIENT_LIGHTS: usize = 4;
const MAX_POINT_LIGHTS: usize = 16;

// 场景光源 Uniform 结构 未来光相关的再抽象一个独立的system
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SceneLightUniforms {
    // 光源数量计数
    num_directional: u32,
    num_ambient: u32,
    num_point: u32,
    _padding: u32,

    // 按类型分组的光源数组
    directional_lights: [DirectionalLightUniform; MAX_DIRECTIONAL_LIGHTS],
    ambient_lights: [AmbientLightUniform; MAX_AMBIENT_LIGHTS],
    // point_lights: [PointLightUniform; MAX_POINT_LIGHTS], // 未来扩展
}

impl Default for SceneLightUniforms {
    fn default() -> Self {
        Self {
            num_directional: 0,
            num_ambient: 0,
            num_point: 0,
            _padding: 0,
            directional_lights: [DirectionalLightUniform::default(); MAX_DIRECTIONAL_LIGHTS],
            ambient_lights: [AmbientLightUniform::default(); MAX_AMBIENT_LIGHTS],
        }
    }
}
use std::hash::{Hash, Hasher};

// 添加Pipeline缓存键
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    pub material_id: MaterialId,
    pub vertex_layout_hash: u64,
    pub env_layout_hash: u64,
}

impl PipelineKey {
    fn new(
        material_id: MaterialId,
        env_vertex_buffer_layout: &[VertexBufferLayout],
        env_pipeline_layouts: &[&wgpu::BindGroupLayout],
    ) -> Self {
        use std::collections::hash_map::DefaultHasher;

        // 计算顶点布局哈希
        let mut hasher = DefaultHasher::new();
        for layout in env_vertex_buffer_layout {
            layout.array_stride.hash(&mut hasher);
            match layout.step_mode {
                wgpu::VertexStepMode::Vertex => 0u8.hash(&mut hasher),
                wgpu::VertexStepMode::Instance => 1u8.hash(&mut hasher),
            }
            for attribute in layout.attributes {
                attribute.offset.hash(&mut hasher);
                attribute.shader_location.hash(&mut hasher);
                format!("{:?}", attribute.format).hash(&mut hasher);
            }
        }
        let vertex_layout_hash = hasher.finish();

        // 计算环境布局哈希
        let mut hasher = DefaultHasher::new();
        for layout in env_pipeline_layouts {
            // 使用布局指针作为唯一标识（假设相同布局会复用对象）
            layout.global_id().hash(&mut hasher);
        }
        let env_layout_hash = hasher.finish();

        PipelineKey {
            material_id,
            vertex_layout_hash,
            env_layout_hash,
        }
    }
}

impl MeshRender {
    pub fn new() -> Self {
        MeshRender {
            pipeline_cache: HashMap::new(),
        }
    }

    pub fn get_all_lights(&self, world: &World) -> LightCollection {
        LightCollection {
            ambient_lights: world.get_entities_with_component::<AmbientLight>(),
            directional_lights: world.get_entities_with_component::<DirectionalLight>(),
            // 未来可以添加点光源、聚光灯等
        }
    }

    // 收集光源绑定信息
    fn collect_light_bindings(scene: &World) -> SceneLightUniforms {
        // 创建场景光源数据结构
        let mut world_lights = SceneLightUniforms::default();

        // 收集方向光
        let directional_entities = scene.get_entities_with_component::<DirectionalLight>();
        world_lights.num_directional =
            directional_entities.len().min(MAX_DIRECTIONAL_LIGHTS) as u32;

        for (i, entity) in directional_entities
            .iter()
            .take(MAX_DIRECTIONAL_LIGHTS)
            .enumerate()
        {
            if let Some(light) = scene.get_component::<DirectionalLight>(*entity) {
                world_lights.directional_lights[i] = DirectionalLightUniform {
                    direction: [
                        light.direction.x,
                        light.direction.y,
                        light.direction.z,
                        0.0, // padding
                    ],
                    color_intensity: [
                        light.light_data.color[0],
                        light.light_data.color[1],
                        light.light_data.color[2],
                        light.light_data.intensity,
                    ],
                    flags: [
                        light.light_data.enabled as u32,
                        light.cast_shadows as u32,
                        0,
                        0,
                    ],
                };
            }
        }

        // 收集环境光
        let ambient_entities = scene.get_entities_with_component::<AmbientLight>();
        world_lights.num_ambient = ambient_entities.len().min(MAX_AMBIENT_LIGHTS) as u32;

        for (i, entity) in ambient_entities.iter().take(MAX_AMBIENT_LIGHTS).enumerate() {
            if let Some(light) = scene.get_component::<AmbientLight>(*entity) {
                world_lights.ambient_lights[i] = AmbientLightUniform {
                    color_intensity: [
                        light.light_data.color[0],
                        light.light_data.color[1],
                        light.light_data.color[2],
                        light.light_data.intensity,
                    ],
                    flags: [light.light_data.enabled as u32, 0, 0, 0],
                };
            }
        }
        world_lights
    }

    /// 获取主相机的绑定信息
    fn get_main_camera_binding(world: &World) -> Option<(u32, &wgpu::Buffer, CameraUniform)> {
        // 首先查找透视相机中的主相机
        for entity in world.get_entities_with_component::<PerspectiveCamera>() {
            if let (Some(camera), Some(gpu_data)) = (
                world.get_component::<PerspectiveCamera>(entity),
                world.get_component::<CameraGpuData>(entity),
            ) {
                if camera.camera_data.is_main {
                    let uniform = camera.to_uniform();
                    return Some((camera.camera_data.bind_index, &gpu_data.buffer, uniform));
                }
            }
        }

        // 然后查找正交相机中的主相机
        for entity in world.get_entities_with_component::<OrthographicCamera>() {
            if let (Some(camera), Some(gpu_data)) = (
                world.get_component::<OrthographicCamera>(entity),
                world.get_component::<CameraGpuData>(entity),
            ) {
                if camera.camera_data.is_main {
                    let uniform = camera.to_uniform();
                    return Some((camera.camera_data.bind_index, &gpu_data.buffer, uniform));
                }
            }
        }

        None
    }
    /// 更新所有相机的 GPU 缓冲区
    fn update_camera_buffers(world: &World, renderer: &Renderer) {
        // 更新透视相机
        for entity in world.get_entities_with_component::<PerspectiveCamera>() {
            if let (Some(camera), Some(gpu_data)) = (
                world.get_component::<PerspectiveCamera>(entity),
                world.get_component::<CameraGpuData>(entity),
            ) {
                let uniform = camera.to_uniform();
                gpu_data.update_buffer(uniform, renderer);
            }
        }

        // 更新正交相机
        for entity in world.get_entities_with_component::<OrthographicCamera>() {
            if let (Some(camera), Some(gpu_data)) = (
                world.get_component::<OrthographicCamera>(entity),
                world.get_component::<CameraGpuData>(entity),
            ) {
                let uniform = camera.to_uniform();
                gpu_data.update_buffer(uniform, renderer);
            }
        }
    }

    fn get_env_bind_groups(world: &World, renderer: &Renderer) -> Vec<EnvBindGroup> {
        let device = &renderer.device;
        let mut env_bind_groups: Vec<EnvBindGroup> = Vec::new();
        let mut bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry> = vec![];
        let mut bind_group_entries: Vec<wgpu::BindGroupEntry> = vec![];

        // 首先更新所有相机的缓冲区
        Self::update_camera_buffers(world, renderer);
        // 添加主相机绑定
        if let Some((bind_index, camera_buffer, _uniform)) = Self::get_main_camera_binding(world) {
            bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: bind_index,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: bind_index,
                resource: camera_buffer.as_entire_binding(),
            });
        }

        let all_lights = Self::collect_light_bindings(world);
        // 创建光源 uniform buffer
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Lights Uniform Buffer"),
            contents: bytemuck::cast_slice(&[all_lights]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 添加光源绑定布局
        bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: LIGHT_BINDING_INDEX, // 光源使用 binding 2（0=相机，1=预留，2=光源）
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        // 添加光源绑定条目
        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: LIGHT_BINDING_INDEX,
            resource: light_buffer.as_entire_binding(),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: bind_group_layout_entries.as_slice(),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &bind_group_entries.as_slice(),
        });
        // env bind group use group 1, user's bind use group 0
        env_bind_groups.push(EnvBindGroup {
            bind_group: bind_group,
            bind_group_layout: bind_group_layout,
            index: ENV_BIND_GROUP_INDEX,
        });
        env_bind_groups
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        view: &wgpu::TextureView,
        world: &mut World,
        renderer: &Renderer,
    ) {
        let env_bind_groups = Self::get_env_bind_groups(world, renderer);
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(world.background_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &renderer.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        let env_pipeline_layouts: &Vec<&wgpu::BindGroupLayout> = &env_bind_groups
            .iter()
            .map(|env_bind_group| &env_bind_group.bind_group_layout)
            .collect();

        let entities = world.get_entities_with_component::<MeshRenderer>();

        self.iter_entities(RenderOptions {
            entities,
            renderer,
            world,
            render_pass,
            env_pipeline_layouts,
            env_bind_groups: &env_bind_groups,
        });
    }

    // 获取或创建pipeline
    fn get_or_create_pipeline(
        &mut self,
        material_id: MaterialId,
        material: &Box<dyn MaterialTrait>,
        renderer: &Renderer,
        env_pipeline_layouts: &Vec<&wgpu::BindGroupLayout>,
        env_vertex_buffer_layout: Vec<VertexBufferLayout>,
    ) -> PipelineKey {
        let key = PipelineKey::new(material_id, &env_vertex_buffer_layout, env_pipeline_layouts);

        // 如果缓存中没有，创建新的pipeline
        if !self.pipeline_cache.contains_key(&key) {
            let pipeline = material.get_render_pipeline(
                renderer,
                env_pipeline_layouts,
                env_vertex_buffer_layout,
            );
            self.pipeline_cache.insert(key.clone(), pipeline);
        }
        key
    }

    pub fn iter_entities(&mut self, option: RenderOptions) {
        let RenderOptions {
            entities,
            renderer,
            world,
            mut render_pass,
            env_pipeline_layouts,
            env_bind_groups,
        } = option;
        let resource_manager = world.resource_manager();

        // 第一步：预先收集所有需要的 pipeline 信息并创建它们
        let mut pipeline_infos = Vec::new();

        for entity in &entities {
            let option_mesh_renderer = world.get_component::<MeshRenderer>(*entity);
            if option_mesh_renderer.is_none() {
                continue;
            }
            let mesh_renderer = option_mesh_renderer.unwrap();

            let mut env_vertex_buffer_layout: Vec<VertexBufferLayout> = Vec::new();
            let option_mesh = resource_manager.get_mesh(mesh_renderer.mesh_id);
            let option_material = resource_manager.get_material(mesh_renderer.material_id);

            if option_mesh.is_none() || option_material.is_none() {
                continue;
            }
            let mesh = option_mesh.unwrap();
            let material = option_material.unwrap();

            // 准备顶点缓冲区布局
            env_vertex_buffer_layout.push(mesh.get_buffer_layout());

            // 检查实例缓冲区
            if let Some(instance) = world.get_component::<Instance>(*entity) {
                if !instance.is_empty() {
                    if let Some(_buffer_id) = resource_manager.get_instance_buffer_id(*entity) {
                        if let Some(layout) = resource_manager.get_buffer_layout_by_name("instance")
                        {
                            env_vertex_buffer_layout.push(layout.layout.clone());
                        }
                    }
                }
            }

            // 预先创建 pipeline
            let pipeline_key = self.get_or_create_pipeline(
                mesh_renderer.material_id,
                &material,
                renderer,
                env_pipeline_layouts,
                env_vertex_buffer_layout,
            );
            pipeline_infos.push((*entity, pipeline_key));
        }

        // 第二步：执行渲染
        for (entity, pipeline_key) in pipeline_infos {
            let option_mesh_renderer = world.get_component::<MeshRenderer>(entity);
            if option_mesh_renderer.is_none() {
                continue;
            }
            let mesh_renderer = option_mesh_renderer.unwrap();

            let mut instance_len = 1;
            let option_mesh = resource_manager.get_mesh(mesh_renderer.mesh_id);
            let option_material = resource_manager.get_material(mesh_renderer.material_id);

            if option_mesh.is_none() || option_material.is_none() {
                continue;
            }
            let mesh = option_mesh.unwrap();
            let material = option_material.unwrap();

            // 获取缓存的 pipeline
            let pipeline = self.pipeline_cache.get(&pipeline_key).unwrap();

            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

            // 处理实例缓冲区
            if let Some(instance) = world.get_component::<Instance>(entity) {
                if !instance.is_empty() {
                    if let Some(buffer_id) = resource_manager.get_instance_buffer_id(entity) {
                        if let Some(buffer_resource) = resource_manager.get_buffer(buffer_id) {
                            render_pass.set_vertex_buffer(1, buffer_resource.buffer.slice(..));
                            instance_len = instance.count() as u32;
                        }
                    }
                }
            }

            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            // 绑定环境 bind group
            for env_bind_group in env_bind_groups {
                render_pass.set_bind_group(env_bind_group.index, &env_bind_group.bind_group, &[]);
            }

            render_pass.set_bind_group(0, material.get_bind_group(), &[]);
            render_pass.set_pipeline(pipeline);

            render_pass.draw_indexed(0..mesh.num_indices, 0, 0..instance_len);
        }
    }
}
