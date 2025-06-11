use anyhow::Result;
use std::io::{BufReader, Cursor};
use tobj::Model;

use crate::{
    components::{mesh_renderer::MeshRenderer, transform::Transform},
    entity::Entity,
    mini_gpu::MiniGPU,
    renderer::Renderer,
    resources::{
        material::blinn_phong::BlinnPhongMaterial,
        mesh::{Mesh, VertexFormat},
        resource_manager::{MaterialId, MeshId},
    },
    world::World,
};

use super::{
    resource::{load_path, load_texture},
    texture::Texture,
};

/// OBJ 模型加载结果
pub struct LoadedModel {
    pub root_entity: Entity,
    pub mesh_entities: Vec<Entity>,
    pub material_ids: Vec<MaterialId>,
    pub mesh_ids: Vec<MeshId>,
}

/// OBJ 加载配置
pub struct ObjLoadConfig {
    pub generate_normals: bool,
    pub flip_uvs: bool,
    pub scale: f32,
    pub position_offset: glam::Vec3,
    pub create_materials: bool,
}

impl Default for ObjLoadConfig {
    fn default() -> Self {
        Self {
            generate_normals: true,
            flip_uvs: false,
            scale: 1.0,
            position_offset: glam::Vec3::ZERO,
            create_materials: true,
        }
    }
}

/// 从文件路径加载 OBJ 模型
pub async fn load_obj(
    path: &std::path::Path,
    mini_gpu: &mut MiniGPU,
    config: Option<ObjLoadConfig>,
) -> Result<LoadedModel> {
    let config = config.unwrap_or_default();

    let obj_text: String = load_path(path).await?;
    let mut obj_reader = BufReader::new(Cursor::new(obj_text));

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mtl_path = path.parent().unwrap().join(&p);
            let mat_text = load_path(&mtl_path).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    load_obj_from_data(
        models,
        obj_materials,
        path.parent().unwrap(),
        &std::collections::HashMap::new(),
        mini_gpu,
        config,
    )
    .await
}

/// 从 URL 和缓冲区映射加载 OBJ 模型
pub async fn load_obj_by_url(
    obj_path: &str,
    dir_buffer_map: &std::collections::HashMap<String, &[u8]>,
    mini_gpu: &mut MiniGPU,
    config: Option<ObjLoadConfig>,
) -> Result<LoadedModel> {
    let config = config.unwrap_or_default();

    let obj_text = dir_buffer_map.get(obj_path).unwrap();
    let mut obj_reader = BufReader::new(Cursor::new(obj_text));

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mtl_text = dir_buffer_map.get(&p).unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mtl_text)))
        },
    )
    .await?;

    load_obj_from_data(
        models,
        obj_materials,
        std::path::Path::new(obj_path).parent().unwrap(),
        dir_buffer_map,
        mini_gpu,
        config,
    )
    .await
}

/// 从解析后的数据创建模型
async fn load_obj_from_data(
    models: Vec<Model>,
    obj_materials: Result<Vec<tobj::Material>, tobj::LoadError>,
    material_path: &std::path::Path,
    dir_buffer_map: &std::collections::HashMap<String, &[u8]>,
    mini_gpu: &mut MiniGPU,
    config: ObjLoadConfig,
) -> Result<LoadedModel> {
    let world = &mut mini_gpu.world_mut();
    // 创建根实体
    let root_entity = world.create_entity();
    world.add_component(root_entity, Transform::default());

    // 加载材质
    let material_ids = if config.create_materials {
        create_materials(obj_materials, material_path, dir_buffer_map, mini_gpu).await?
    } else {
        Vec::new()
    };

    // 加载网格和创建子实体
    let (mesh_entities, mesh_ids) =
        create_mesh_entities(models, &material_ids, mini_gpu, root_entity, &config)?;

    Ok(LoadedModel {
        root_entity,
        mesh_entities,
        material_ids,
        mesh_ids,
    })
}

/// 创建材质并注册到资源管理器
async fn create_materials(
    obj_materials: Result<Vec<tobj::Material>, tobj::LoadError>,
    material_path: &std::path::Path,
    dir_buffer_map: &std::collections::HashMap<String, &[u8]>,
    mini_gpu: &mut MiniGPU,
) -> Result<Vec<MaterialId>> {
    let mut material_ids = Vec::new();

    match obj_materials {
        Ok(materials) => {
            for material in materials {
                let material_id =
                    create_material_from_obj(material, material_path, dir_buffer_map, mini_gpu)
                        .await?;
                material_ids.push(material_id);
            }
        }
        Err(e) => {
            // 如果材质加载失败，创建默认材质
            println!(
                "Warning: Failed to load materials: {:?}, using default material",
                e
            );
            let default_material_id = create_default_material(mini_gpu)?;
            material_ids.push(default_material_id);
        }
    }

    // 如果没有材质，至少创建一个默认材质
    if material_ids.is_empty() {
        let default_material_id = create_default_material(mini_gpu)?;
        material_ids.push(default_material_id);
    }

    Ok(material_ids)
}

/// 从 OBJ 材质创建材质
async fn create_material_from_obj(
    obj_material: tobj::Material,
    material_path: &std::path::Path,
    dir_buffer_map: &std::collections::HashMap<String, &[u8]>,
    mini_gpu: &mut MiniGPU,
) -> Result<MaterialId> {
    let mut material_config = crate::resources::material::blinn_phong::BlinnPhongMaterialConfig {
        name: obj_material.name.clone(),
        ..Default::default()
    };
    let device = &mini_gpu.renderer().device;
    let queue = &mini_gpu.renderer().queue;

    // 加载漫反射纹理
    if let Some(diffuse_texture_name) = obj_material.diffuse_texture {
        let diffuse_path = material_path.join(&diffuse_texture_name);
        let diffuse_path_string = diffuse_path.to_str().unwrap();

        let diffuse_texture = if let Some(buffer) = dir_buffer_map.get(diffuse_path_string) {
            Texture::from_bytes(device, queue, buffer, diffuse_path_string)?
        } else {
            load_texture(diffuse_path_string, device, queue).await?
        };

        material_config.diffuse_texture = Some(diffuse_texture);
    }

    // 设置材质属性
    let diffuse_color = obj_material.diffuse.unwrap_or([1.0, 1.0, 1.0]); // 默认白色
    material_config.diffuse_color =
        glam::Vec3::new(diffuse_color[0], diffuse_color[1], diffuse_color[2]);

    let specular_color = obj_material.specular.unwrap_or([1.0, 1.0, 1.0]); // 默认白色
    material_config.specular_color =
        glam::Vec3::new(specular_color[0], specular_color[1], specular_color[2]);

    material_config.shininess = obj_material.shininess.unwrap_or(32.0); // 默认值

    // 创建材质并注册到资源管理器
    let material = BlinnPhongMaterial::new(material_config, mini_gpu.renderer());
    let material_id = mini_gpu
        .world_mut()
        .resource_manager_mut()
        .add_material(Box::new(material), Some(obj_material.name));

    Ok(material_id)
}

/// 创建默认材质
fn create_default_material(mini_gpu: &mut MiniGPU) -> Result<MaterialId> {
    let default_config = crate::resources::material::blinn_phong::BlinnPhongMaterialConfig {
        name: "DefaultMaterial".to_string(),
        diffuse_color: [0.8, 0.8, 0.8].into(),
        specular_color: [0.2, 0.2, 0.2].into(),
        shininess: 32.0,
        ..Default::default()
    };

    let material = BlinnPhongMaterial::new(default_config, mini_gpu.renderer());
    let material_id = mini_gpu
        .world_mut()
        .resource_manager_mut()
        .add_material(Box::new(material), Some("DefaultMaterial".to_string()));

    Ok(material_id)
}

/// 创建网格实体
fn create_mesh_entities(
    models: Vec<Model>,
    material_ids: &[MaterialId],
    mini_gpu: &mut MiniGPU,
    parent_entity: Entity,
    config: &ObjLoadConfig,
) -> Result<(Vec<Entity>, Vec<MeshId>)> {
    let mut mesh_entities = Vec::new();
    let mut mesh_ids = Vec::new();
    for (model_index, model) in models.into_iter().enumerate() {
        let material_id = if model.mesh.material_id.is_some() {
            model.mesh.material_id.unwrap_or(0)
        } else {
            0 // 默认材质 ID
        };
        // 创建网格
        let mesh = build_mesh_from_obj(model.mesh, mini_gpu, config)?;
        let world = mini_gpu.world_mut();

        let mesh_id = world
            .resource_manager_mut()
            .add_mesh(mesh, Some(model.name.clone()));
        mesh_ids.push(mesh_id);

        // 创建实体
        let entity = world.create_entity();

        // 添加变换组件
        let mut transform = Transform::default();
        transform.position = config.position_offset;
        transform.scale = glam::Vec3::splat(config.scale);
        world.add_component(entity, transform);

        // 添加网格渲染器组件
        let material_id = if !material_ids.is_empty() {
            material_ids[material_id.min(material_ids.len() - 1)]
        } else {
            // 如果没有材质，需要创建一个默认材质
            create_default_material(mini_gpu)?
        };

        let mesh_renderer = MeshRenderer {
            mesh_id,
            material_id,
            ..Default::default()
        };
        // 重新拿world, 因为需要确保mini_gpu传到其他函数时，没有引用其内部数据，避免编译器报错
        mini_gpu.world_mut().add_component(entity, mesh_renderer);

        mesh_entities.push(entity);
    }

    Ok((mesh_entities, mesh_ids))
}

/// 从 OBJ 网格数据构建 Mesh
fn build_mesh_from_obj(
    obj_mesh: tobj::Mesh,
    mini_gpu: &mut MiniGPU,
    config: &ObjLoadConfig,
) -> Result<Mesh> {
    // 检查数据完整性
    let has_normals = !obj_mesh.normals.is_empty();
    let has_texcoords = !obj_mesh.texcoords.is_empty();

    // 确定顶点格式
    let vertex_format = match (has_normals, has_texcoords) {
        (true, true) => VertexFormat::PositionNormalTexture,
        (true, false) => VertexFormat::PositionNormal,
        (false, true) => VertexFormat::PositionTexture,
        (false, false) => VertexFormat::PositionOnly,
    };

    // 构建顶点数据
    let vertices = build_vertex_data(&obj_mesh, vertex_format, config)?;

    // 应用配置变换
    let indices = obj_mesh.indices;
    if config.flip_uvs {
        // 如果需要翻转 UV，可以在这里处理
    }

    // 创建 Mesh
    let mesh = Mesh::new(
        bytemuck::cast_slice(&vertices),
        indices,
        vertex_format,
        mini_gpu.renderer(),
    );

    Ok(mesh)
}

/// 构建顶点数据
fn build_vertex_data(
    obj_mesh: &tobj::Mesh,
    vertex_format: VertexFormat,
    config: &ObjLoadConfig,
) -> Result<Vec<f32>> {
    let vertex_count = obj_mesh.positions.len() / 3;
    let mut vertices = Vec::new();

    for i in 0..vertex_count {
        // 位置
        let pos_index = i * 3;
        vertices.extend_from_slice(&[
            obj_mesh.positions[pos_index] * config.scale + config.position_offset.x,
            obj_mesh.positions[pos_index + 1] * config.scale + config.position_offset.y,
            obj_mesh.positions[pos_index + 2] * config.scale + config.position_offset.z,
        ]);

        // 法线（如果存在）
        if matches!(
            vertex_format,
            VertexFormat::PositionNormal | VertexFormat::PositionNormalTexture
        ) {
            if !obj_mesh.normals.is_empty() {
                let norm_index = i * 3;
                vertices.extend_from_slice(&[
                    obj_mesh.normals[norm_index],
                    obj_mesh.normals[norm_index + 1],
                    obj_mesh.normals[norm_index + 2],
                ]);
            } else if config.generate_normals {
                // 如果需要生成法线，这里可以添加法线生成逻辑
                vertices.extend_from_slice(&[0.0, 1.0, 0.0]); // 临时默认法线
            }
        }

        // 纹理坐标（如果存在）
        if matches!(
            vertex_format,
            VertexFormat::PositionTexture | VertexFormat::PositionNormalTexture
        ) {
            if !obj_mesh.texcoords.is_empty() {
                let tex_index = i * 2;
                let u = obj_mesh.texcoords[tex_index];
                let mut v = obj_mesh.texcoords[tex_index + 1];

                if config.flip_uvs {
                    v = 1.0 - v;
                }

                vertices.extend_from_slice(&[u, v]);
            } else {
                vertices.extend_from_slice(&[0.0, 0.0]); // 默认纹理坐标
            }
        }
    }

    Ok(vertices)
}

/// 便利函数：加载简单的 OBJ 模型
pub async fn load_simple_obj(path: &str, mini_gpu: &mut MiniGPU) -> Result<Entity> {
    let path = std::path::Path::new(path);
    let loaded_model = load_obj(path, mini_gpu, None).await?;
    Ok(loaded_model.root_entity)
}

/// 便利函数：加载带自定义配置的 OBJ 模型
pub async fn load_obj_with_config(
    path: &str,
    scale: f32,
    position: glam::Vec3,
    mini_gpu: &mut MiniGPU,
) -> Result<Entity> {
    let config = ObjLoadConfig {
        scale,
        position_offset: position,
        ..Default::default()
    };

    let path = std::path::Path::new(path);
    let loaded_model = load_obj(path, mini_gpu, Some(config)).await?;
    Ok(loaded_model.root_entity)
}
