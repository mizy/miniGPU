use crate::{
    components::{
        material::{self, Material},
        mesh::Mesh,
    },
    renderer::Renderer,
    scene::{self, Scene},
};

use super::system::System;

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
        {
            let mut render_pass = {
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                })
            };
            scene.entities.iter().for_each(|entity| {
                if !entity.has_component("mesh") || !entity.has_component("material") {
                    return;
                }
                let mesh = scene.get_entity_component::<Mesh>(&entity, "mesh");
                let material = scene.get_entity_component::<Material>(&entity, "material");
                render_pass.set_pipeline(&material.pipeline);
                render_pass.set_bind_group(0, &material.bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
            });
        }
        renderer.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
impl MeshRender {}
