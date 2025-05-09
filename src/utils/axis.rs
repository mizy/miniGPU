use crate::{
    components::{
        material::{Material, MaterialConfig, MaterialTrait},
        materials::shader::ShaderParser,
        mesh::Mesh,
    },
    entity,
    mini_gpu::MiniGPU,
};

const TEST_XYZLINE_SHADER: &str = r#"
#include <CameraUniform>

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) position: vec3<f32>,
}
@vertex
fn vs_main(@location(0) pos: vec3f) -> VertexOutput {
  var clip_position = camera.projection_matrix * camera.view_matrix * vec4f(pos, 1.);
  var out: VertexOutput;
  out.position = pos;
  out.clip_position = clip_position;
  return out;
}

@fragment
fn fs_main(out:VertexOutput) -> @location(0) vec4f {
  return vec4f(out.position, 1.);
}
"#;

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
                .parse_shader(TEST_XYZLINE_SHADER)
                .to_string()
                .to_string(),
            topology: wgpu::PrimitiveTopology::LineList,
            uniforms: vec![0.],
        },
        &mini_gpu.renderer,
    );
    let entity_id = mini_gpu.scene.add_entity(entity::Entity::new());
    mini_gpu.scene.set_entity_component(entity_id, mesh, "mesh");
    mini_gpu
        .scene
        .set_entity_component::<Box<dyn MaterialTrait>>(
            entity_id,
            Box::new(material_line),
            "material",
        );
}
