use wgpu::CommandEncoder;

use crate::{
    components::{
        material::{Material, MaterialRef, MaterialTrait},
        mesh::Mesh,
        perspectivecamera::PerspectiveCamera,
    },
    renderer::Renderer,
    scene::Scene,
};

use super::system::System;

pub struct EnvBindGroup<'a> {
    pub bind_group: &'a wgpu::BindGroup,
    pub bind_group_layout: &'a wgpu::BindGroupLayout,
    pub index: u32,
}

pub struct MeshRender {}
impl System for MeshRender {
    fn update(&self, renderer: &Renderer, scene: &Scene) {
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
        let mut env_bind_groups: Vec<EnvBindGroup> = vec![];

        // add camera bind group
        let camera: Option<&mut PerspectiveCamera> = scene.get_camera_mut();
        if let Some(camera_val) = camera {
            env_bind_groups.push(EnvBindGroup {
                bind_group: camera_val.get_bind_group(),
                bind_group_layout: camera_val.get_bind_group_layout(),
                index: 1,
            });
        }

        Self::render(&mut encoder, &view, scene, renderer, &env_bind_groups);
        renderer.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

impl MeshRender {
    pub fn render(
        encoder: &mut CommandEncoder,
        view: &wgpu::TextureView,
        scene: &Scene,
        renderer: &Renderer,
        env_bind_groups: &Vec<EnvBindGroup>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(scene.background_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        let env_pipeline_layouts: Vec<&wgpu::BindGroupLayout> = env_bind_groups
            .iter()
            .map(|env_bind_group| env_bind_group.bind_group_layout)
            .collect();

        scene.entities.iter().for_each(|entity| {
            if !entity.has_component("mesh") || !entity.has_component("material") {
                return;
            }
            let mesh = scene.get_entity_component::<Mesh>(&entity, "mesh");
            let material = scene.get_entity_component::<MaterialRef>(&entity, "material");
            let mateiral_mut =
                unsafe { &mut *(material as *const MaterialRef as *mut MaterialRef) };

            render_pass.set_bind_group(0, material.get_bind_group(), &[]);
            render_pass
                .set_pipeline(mateiral_mut.get_render_pipeline(renderer, &env_pipeline_layouts));
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            for env_bind_group in env_bind_groups {
                render_pass.set_bind_group(env_bind_group.index, env_bind_group.bind_group, &[]);
            }

            render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
        });
    }
}
