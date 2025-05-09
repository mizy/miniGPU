use glam::Vec3;
// todo: make a shader computer mesh line, for 2d render in 3d space
use crate::{
    components::{
        material::{Material, MaterialConfig, MaterialTrait},
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
    let (vertices, indices) = make_width_line_vertexes(path, width);
    println!("vertices: {:?}", vertices);
    let mesh = Mesh::new_position_only(vertices, indices, renderer);
    scene.set_entity_component(entity_id, mesh, "mesh");
}

pub fn make_width_line_vertexes(path: &Vec<Vec3>, width: f32) -> (Vec<f32>, Vec<u32>) {
    let mut vertexes = vec![];
    let mut indices = vec![];
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
        indices.push(i as u32 * 2);
        vertexes.push(position.x - normal.x);
        vertexes.push(position.y - normal.y);
        vertexes.push(position.z - normal.z);
        indices.push(i as u32 * 2 + 1);
    }
    (vertexes, indices)
}

pub fn make_material(renderer: &Renderer, scene: &mut Scene, color: Vec<f32>, entity_id: usize) {
    let shader = include_str!("../components/materials/shaders/default.wgsl");
    let mut shader_parser = crate::components::materials::shader::ShaderParser::new();
    let shader = shader_parser.parse_shader(shader);
    let material = Material::new(
        MaterialConfig {
            shader,
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            uniforms: color,
        },
        renderer,
    );
    // material.pipeline.
    scene.set_entity_component::<Box<dyn MaterialTrait>>(entity_id, Box::new(material), "material");
}
