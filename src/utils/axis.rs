use crate::{
    mini_gpu::MiniGPU,
    resources::shaders::shader::ShaderParser,
    resources::{
        material::{Material, MaterialConfig},
        mesh::Mesh,
    },
};

pub fn add_xyz_line(mini_gpu: &mut MiniGPU, user_size: Option<f32>) {
    let size = user_size.unwrap_or(0.5);
    let mesh = Mesh::new_position_only(
        vec![
            0.0, 0.0, 0.0, size, 0.0, 0.0, 0.0, size, 0.0, 0.0, 0.0, size,
        ],
        vec![0, 1, 0, 2, 0, 3],
        &mini_gpu.renderer,
    );
    let mut shader_parser = ShaderParser::new();

    let material_line = Material::new(
        MaterialConfig {
            shader: shader_parser
                .parse_shader(include_str!("../resources/shaders/axis.wgsl"))
                .to_string(),
            topology: wgpu::PrimitiveTopology::LineList,
            uniforms: vec![0.],
        },
        &mini_gpu.renderer,
    );
    let entity_id = mini_gpu.world.create_entity();
    mini_gpu.world.add_component(entity_id, mesh);
    mini_gpu
        .world
        .add_component(entity_id, Box::new(material_line));
}
