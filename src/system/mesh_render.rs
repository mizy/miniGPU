use wgpu::{CommandEncoder, StoreOp, VertexBufferLayout};

use crate::{
    components::{
        instance::Instance, lights::light::LightTrait, material::MaterialTrait, mesh::Mesh,
        perspective_camera::CameraTrait,
    },
    entity::Entity,
    renderer::Renderer,
    scene::Scene,
};

use super::system::System;

pub struct EnvBindGroup {
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub index: u32,
}

pub struct RenderOptions<'a> {
    entities: &'a Vec<Entity>,
    renderer: &'a Renderer,
    scene: &'a Scene,
    render_pass: wgpu::RenderPass<'a>,
    env_pipeline_layouts: &'a Vec<&'a wgpu::BindGroupLayout>,
    env_bind_groups: &'a Vec<EnvBindGroup>,
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

        Self::render(&mut encoder, &view, scene, renderer);
        renderer.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

impl MeshRender {
    fn get_env_bind_groups(scene: &Scene, renderer: &Renderer) -> Vec<EnvBindGroup> {
        let device = &renderer.device;
        let mut env_bind_groups: Vec<EnvBindGroup> = Vec::new();
        let mut bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry> = vec![];
        let mut bind_group_entries: Vec<wgpu::BindGroupEntry> = vec![];

        // join all bind groups to 1 bind group
        // add camera bind group
        let camera: Option<&mut Box<dyn CameraTrait>> = scene.get_default_camera();
        if let Some(camera_val) = camera {
            bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: camera_val.get_bind_index(),
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: camera_val.get_bind_index(),
                resource: camera_val.get_buffer().as_entire_binding(),
            });
        }

        // add lights bind group
        scene.entities.iter().for_each(|entity| {
            // each entity has only a light component
            if !entity.has_component("light") {
                return;
            }
            let light = scene.get_entity_component::<Box<dyn LightTrait>>(&entity, "light");
            bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: light.get_bind_index(),
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: light.get_bind_index(),
                resource: light.get_buffer().as_entire_binding(),
            });
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: bind_group_layout_entries.as_slice(),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &bind_group_entries.as_slice(),
        });
        // env bind group use group 1, user's bind use group 0
        env_bind_groups.push(EnvBindGroup {
            bind_group: bind_group,
            bind_group_layout: bind_group_layout,
            index: 1,
        });
        env_bind_groups
    }

    pub fn render(
        encoder: &mut CommandEncoder,
        view: &wgpu::TextureView,
        scene: &Scene,
        renderer: &Renderer,
    ) {
        let env_bind_groups = Self::get_env_bind_groups(scene, renderer);
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(scene.background_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &renderer.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        let env_pipeline_layouts: &Vec<&wgpu::BindGroupLayout> = &env_bind_groups
            .iter()
            .map(|env_bind_group| &env_bind_group.bind_group_layout)
            .collect();

        let entities = &scene.entities;

        Self::iter_entities(RenderOptions {
            entities,
            renderer,
            scene,
            render_pass,
            env_pipeline_layouts,
            env_bind_groups: &env_bind_groups,
        });
    }

    pub fn iter_entities(option: RenderOptions) {
        let RenderOptions {
            entities,
            renderer,
            scene,
            mut render_pass,
            env_pipeline_layouts,
            env_bind_groups,
        } = option;
        for entity in entities {
            if !entity.has_component("mesh") || !entity.has_component("material") {
                continue;
            }
            let mut env_vertex_buffer_layout: Vec<VertexBufferLayout> = Vec::new();
            let mut instance_len = 1;
            let mesh = scene.get_entity_component::<Mesh>(&entity, "mesh");
            let material =
                scene.get_entity_component::<Box<dyn MaterialTrait>>(&entity, "material");
            let mateiral_mut =
                scene.get_entity_component_mut::<Box<dyn MaterialTrait>>(&entity, "material");

            // bind mesh
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            env_vertex_buffer_layout.push(mesh.get_buffer_layout());

            // bind instance buffer
            if entity.has_component("instance") {
                let instance = scene.get_entity_component::<Instance>(&entity, "instance");
                render_pass
                    .set_vertex_buffer(Instance::get_buffer_index(), instance.buffer.slice(..));
                env_vertex_buffer_layout.push(Instance::get_buffer_layout());
                instance_len = instance.data.len() as u32
            }

            // set index buffer
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            // bind env bind group
            for env_bind_group in env_bind_groups {
                render_pass.set_bind_group(env_bind_group.index, &env_bind_group.bind_group, &[]);
            }

            // set pipeline and bind group layout
            render_pass.set_bind_group(0, material.get_bind_group(), &[]);
            render_pass.set_pipeline(mateiral_mut.get_render_pipeline(
                renderer,
                env_pipeline_layouts,
                env_vertex_buffer_layout,
            ));

            render_pass.draw_indexed(0..mesh.num_indices, 0, 0..instance_len);
        }
    }
}
