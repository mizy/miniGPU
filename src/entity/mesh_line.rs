use glam::Vec3;
// todo: make a shader computer mesh line, for 2d render in 3d space
use crate::{
    components::{
        material::MaterialTrait,
        materials::sprite::{SpriteMaterial, SpriteMaterialConfig},
        mesh::Mesh,
    },
    renderer::Renderer,
    scene::Scene,
};

pub fn make_mesh(
    path: &Vec<Vec3>,
    width: f32,
    renderer: &Renderer,
    scene: &mut Scene,
    entity_id: usize,
) {
    let vertices = make_width_line_vertexes(path, width);
    let mesh = Mesh::new(vertices, vec![0, 1, 2, 2, 1, 3], renderer);
    scene.set_entity_component(entity_id, mesh, "mesh");
}

pub fn make_width_line_vertexes(path: &Vec<Vec3>, width: f32) -> Vec<f32> {
    let mut vertexes = vec![];
    let half_width = width / 2.0;
    // calculate line
    for i in 0..path.len() {
        let before_position = if i == 0 { path[i] } else { path[i - 1] };
        let next_position = if i == path.len() - 1 {
            path[i]
        } else {
            path[i + 1]
        };
        let direction = (next_position - before_position).normalize();
        let normal = Vec3::new(-direction.y, direction.x, 0.0);
        let normal = normal * half_width;
        let position = path[i];
        vertexes.push(position.x + normal.x);
        vertexes.push(position.y + normal.y);
        vertexes.push(position.z + normal.z);
        vertexes.push(position.x - normal.x);
        vertexes.push(position.y - normal.y);
        vertexes.push(position.z - normal.z);
    }
    vertexes
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
