use std::vec;

use ::mini_gpu::{
    components::{
        materials::image::{Image, ImageConfig},
        mesh::Mesh,
    },
    entity::{self, Entity},
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
};
use bytemuck::{Pod, Zeroable};
use mini_gpu::mini_gpu;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
fn main() {
    pollster::block_on(run());
    print!("Hello, world!");
}

async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    let size = window.inner_size();
    let mut mini_gpu = mini_gpu::MiniGPU::new(
        mini_gpu::MiniGPUConfig {
            width: size.width,
            height: size.height,
        },
        window,
    )
    .await;
    make_test_mesh(&mut mini_gpu);
    mini_gpu
        .scene
        .add_system("render".to_string(), Box::new(MeshRender {}));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let window = &mini_gpu.renderer.window;
        match event {
            Event::RedrawRequested(_) => {
                if let Err(e) = mini_gpu.renderer.render(&mini_gpu.scene) {
                    println!("Failed to render: {}", e);
                }
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        mini_gpu
                            .renderer
                            .resize(physical_size.width, physical_size.height);
                        mini_gpu.renderer.window.request_redraw();
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size 是 &&mut 类型，因此需要解引用两次
                        mini_gpu
                            .renderer
                            .resize(new_inner_size.width, new_inner_size.height);
                        mini_gpu.renderer.window.request_redraw();
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    let vertices = vec![
        Vertex {
            position: [-0.5, -0.5],
            tex_coords: [0., 1. - 0.],
        },
        Vertex {
            position: [0.5, -0.5],
            tex_coords: [1., 1. - 0.],
        },
        Vertex {
            position: [0.5, 0.5],
            tex_coords: [1., 1. - 1.],
        },
        Vertex {
            position: [-0.5, 0.5],
            tex_coords: [0., 1. - 1.],
        },
    ];
    let indices = vec![0, 1, 2, 2, 3, 0];
    let vertex_buffer =
        mini_gpu
            .renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
    let index_buffer =
        mini_gpu
            .renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
    let mesh = Mesh {
        vertex_buffer,
        index_buffer,
        num_indices: indices.len() as u32,
    };
    let image = image::load_from_memory(include_bytes!("./test.png")).unwrap();
    let material = Image::new(
        ImageConfig {
            width: image.width(),
            height: image.height(),
            diffuse_data: image.to_rgba8().into_raw(),
        },
        &mini_gpu.renderer,
    );
    println!("width: {}", image.width());
    println!("height: {}", image.height());

    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu.scene.set_entity_component(entity_id, mesh, "mesh");
    mini_gpu
        .scene
        .set_entity_component(entity_id, material, "material");
}
