use std::io::{BufReader, Cursor};

use anyhow::Ok;
use tobj::Model;
use wgpu::util::DeviceExt;

use crate::{
    components::{material::MaterialTrait, materials, mesh::Mesh},
    entity::Entity,
    mini_gpu::MiniGPU,
    renderer::Renderer,
};

use super::{
    resource::{load_path, load_texture},
    texture::Texture,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}

pub async fn load_obj(path: &std::path::Path, mini_gpu: &mut MiniGPU) -> anyhow::Result<usize> {
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

    // check obj_materials result
    let materials = make_material_map(
        path.parent().unwrap(),
        obj_materials,
        &std::collections::HashMap::new(),
        &mini_gpu.renderer,
    )
    .await?;

    let parent_id = &mini_gpu.scene.add_default_entity();

    append_mesh_children(*parent_id, mini_gpu, models, materials);
    Ok(*parent_id)
}

pub async fn load_obj_by_url(
    obj_path: &str,
    dir_buffer_map: &std::collections::HashMap<String, &[u8]>,
    mini_gpu: &mut MiniGPU,
) -> anyhow::Result<usize> {
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

    // check obj_materials result
    let materials = make_material_map(
        std::path::Path::new(obj_path).parent().unwrap(),
        obj_materials,
        dir_buffer_map,
        &mini_gpu.renderer,
    )
    .await?;

    let parent_id = &mini_gpu.scene.add_default_entity();

    append_mesh_children(*parent_id, mini_gpu, models, materials);
    Ok(*parent_id)
}

pub async fn make_material_map<'a>(
    material_path: &'a std::path::Path,
    obj_materials: Result<Vec<tobj::Material>, tobj::LoadError>,
    dir_buffer_map: &'a std::collections::HashMap<String, &[u8]>,
    renderer: &Renderer,
) -> anyhow::Result<Vec<Box<dyn MaterialTrait>>> {
    let mut materials: Vec<Box<dyn MaterialTrait>> = Vec::new();
    for m in obj_materials? {
        let m_string = match m.diffuse_texture {
            Some(ref name) => name,
            None => {
                continue;
            }
        };

        let diffuse_path = material_path.join(m_string);
        let diffuse_path_string = diffuse_path.to_str().unwrap();
        // use different material here,but we now only have image material

        let diffuse_buffer = dir_buffer_map.get(diffuse_path_string);
        let diffuse_texture: Texture = {
            if let Some(buffer) = diffuse_buffer {
                Texture::from_bytes(
                    &renderer.device,
                    &renderer.queue,
                    &buffer,
                    diffuse_path_string,
                )?
            } else {
                load_texture(diffuse_path_string, &renderer.device, &renderer.queue).await?
            }
        };

        let material = materials::blinn_phong::BlinnPhongMaterial::new(
            materials::blinn_phong::BlinnPhongMaterialConfig {
                name: m.name,
                diffuse_texture: Some(diffuse_texture),
                ..Default::default()
            },
            &renderer,
        );
        materials.push(Box::new(material));
        break;
    }
    Ok(materials)
}

pub fn append_mesh_children(
    parent: usize,
    mini_gpu: &mut MiniGPU,
    models: Vec<Model>,
    materials: Vec<Box<dyn MaterialTrait>>,
) -> usize {
    let mut i = 0;
    let material_ids: Vec<usize> = materials
        .into_iter()
        .map(|m| mini_gpu.scene.add_component(m))
        .collect();
    models.into_iter().for_each(|model| {
        let material_index = material_ids[model.mesh.material_id.unwrap_or(0)];
        let mesh = build_mesh(&mini_gpu.renderer, model.mesh);
        let mut child = Entity::new();
        child.name = model.name;
        let mesh_index = mini_gpu.scene.add_component(mesh);
        child.set_component_index("mesh", mesh_index);
        child.set_component_index("material", material_index);
        let _ = &mini_gpu.scene.add_entity_child(parent, child);
        i += 1;
    });
    parent
}

pub fn build_mesh(renderer: &Renderer, mesh: tobj::Mesh) -> Mesh {
    let mut vertices: Vec<f32> = Vec::new();
    (0..mesh.positions.len() / 3).for_each(|i| {
        vertices.append(&mut vec![
            mesh.positions[i * 3],
            mesh.positions[i * 3 + 1],
            mesh.positions[i * 3 + 2],
            mesh.texcoords[i * 2],
            mesh.texcoords[i * 2 + 1],
            mesh.normals[i * 3],
            mesh.normals[i * 3 + 1],
            mesh.normals[i * 3 + 2],
        ])
    });
    // let mut mesh = Mesh::new(vertices, indices, renderer);
    let vertex_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
    let index_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
    Mesh {
        vertex_buffer,
        index_buffer,
        num_indices: mesh.indices.len() as u32,
        vertex_buffer_layout: get_buffer_layout(),
    }
}

pub fn get_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    use std::mem;
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x3,
            },
        ],
    }
}
