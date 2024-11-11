use anyhow::Ok;
use gltf::{buffer::Data, iter::Materials, Document};
use wgpu::util::DeviceExt;

use crate::{
    components::{material::MaterialTrait, materials, mesh::Mesh},
    entity::Entity,
    mini_gpu::MiniGPU,
    renderer::Renderer,
};

use super::texture::Texture;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}

pub async fn load_gltf(glb_model: &[u8], mini_gpu: &mut MiniGPU) -> anyhow::Result<usize> {
    // check obj_materials result
    let (gltf, buffers, images) = gltf::import_slice(glb_model)?;
    let materials = gltf.materials();
    let materials_map = make_material_map(materials, images, &mini_gpu.renderer)?;

    let parent_id = &mini_gpu.scene.add_default_entity();
    append_mesh_children(*parent_id, mini_gpu, &gltf, &buffers, &materials_map);
    Ok(*parent_id)
}

pub fn make_material_map<'a>(
    materials: Materials,
    images: Vec<gltf::image::Data>,
    renderer: &Renderer,
) -> anyhow::Result<Vec<Box<dyn MaterialTrait>>> {
    let mut material_vec: Vec<Box<dyn MaterialTrait>> = Vec::new();

    for material in materials {
        let mut diffuse_texture: Option<Texture> = None;

        // 检查是否有基础颜色纹理
        if let Some(texture_info) = material.pbr_metallic_roughness().base_color_texture() {
            let texture = texture_info.texture();
            let image_index = texture.source().index();
            let image_data = &images[image_index];

            // 创建纹理
            diffuse_texture = Some(Texture::from_bytes(
                &renderer.device,
                &renderer.queue,
                &image_data.pixels,
                &texture.name().unwrap_or("Unnamed texture").to_string(),
            )?);
        }

        // 创建材质
        let material = materials::image::Image::new(
            materials::image::ImageConfig {
                name: material.name().unwrap_or("Unnamed image").to_string(),
                width: diffuse_texture.as_ref().map_or(1, |t| t.size.width),
                height: diffuse_texture.as_ref().map_or(1, |t| t.size.height),
                texture: diffuse_texture,
                ..Default::default()
            },
            &renderer,
        );

        material_vec.push(Box::new(material));
    }

    Ok(material_vec)
}

pub fn append_mesh_children(
    parent: usize,
    mini_gpu: &mut MiniGPU,
    model: &Document,
    buffers: &Vec<gltf::buffer::Data>,
    materials: &Vec<Box<dyn MaterialTrait>>,
) -> usize {
    let material_ids: Vec<usize> = materials
        .into_iter()
        .map(|m| mini_gpu.scene.add_component(m))
        .collect();
    let meshs = model.meshes();
    meshs.into_iter().for_each(|mesh| {
        let mesh_group = build_group_mesh(&mesh, mini_gpu, &buffers, &material_ids);
        mini_gpu.scene.add_entity_child(parent, mesh_group); //添加到父节点
    });
    parent
}

pub fn build_group_mesh(
    mesh: &gltf::Mesh,
    mini_gpu: &mut MiniGPU,
    buffers: &Vec<Data>,
    material_ids: &Vec<usize>,
) -> Entity {
    let i = 0;
    let mut group = Entity::new();
    for primitive in mesh.primitives() {
        let mesh_instance = build_mesh(&mini_gpu.renderer, &primitive, &buffers);
        let mut child = Entity::new();
        child.name = format!("{}-primitive-{}", mesh.name().unwrap_or("Unnamed mesh"), i);
        let primitive_mateiral_index = primitive.material().index().unwrap_or(0);
        let material_index = material_ids[primitive_mateiral_index];
        let mesh_index = mini_gpu.scene.add_component(mesh_instance);
        child.set_component_index("mesh", mesh_index);
        child.set_component_index("material", material_index);

        let child_id = mini_gpu.scene.add_entity(child);
        group.add_child(child_id);
    }
    group
}

pub fn build_mesh(renderer: &Renderer, primitive: &gltf::Primitive, buffers: &Vec<Data>) -> Mesh {
    let mut vertices: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    let positions = reader.read_positions().unwrap();
    let normals = reader.read_normals().unwrap();
    let tex_coords = reader.read_tex_coords(0).unwrap().into_f32();

    let indices_iter = reader.read_indices().unwrap().into_u32();
    positions
        .zip(normals)
        .zip(tex_coords)
        .for_each(|((position, normal), tex_coord)| {
            vertices.extend_from_slice(&position);
            vertices.extend_from_slice(&tex_coord);
            vertices.extend_from_slice(&normal);
        });
    indices.extend(indices_iter);
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
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
    Mesh {
        vertex_buffer,
        index_buffer,
        num_indices: indices.len() as u32,
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
                format: wgpu::VertexFormat::Float32x3, // position
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2, // tex_coord
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x3, // normal
            },
        ],
    }
}
