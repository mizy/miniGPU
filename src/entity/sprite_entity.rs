use glam::Vec3;

use crate::{
    components::{
        material::MaterialTrait,
        materials::{
            image::{Image, ImageConfig},
            shader::ShaderParser,
            sprite::{SpriteMaterial, SpriteMaterialConfig},
        },
        mesh::Mesh,
    },
    renderer::Renderer,
    scene::Scene,
};

pub fn make_mesh(position: Vec3, renderer: &Renderer, scene: &mut Scene, entity_id: usize) {
    let mesh = Mesh::new(
        vec![
            position.x, position.y, position.z, position.x, position.y, position.z, position.x,
            position.y, position.z, position.x, position.y, position.z,
        ],
        vec![0, 1, 2, 2, 1, 3],
        renderer,
    );
    scene.set_entity_component(entity_id, mesh, "mesh");
}

pub fn make_material(
    renderer: &Renderer,
    scene: &mut Scene,
    material: SpriteMaterialConfig,
    entity_id: usize,
) {
    let material = SpriteMaterial::new(material, renderer);
    scene.set_entity_component::<Box<dyn MaterialTrait>>(entity_id, Box::new(material), "material");
}
