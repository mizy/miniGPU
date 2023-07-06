use ::mini_gpu::{
    components::{
        self,
        material::{self, Material, MaterialConfig},
        mesh::Mesh,
    },
    entity::{self, Entity},
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
};
use mini_gpu::mini_gpu;
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

fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    let mesh = Mesh::new(
        vec![0.5, 0.5, -0.5, 0.5, 0., 0.],
        vec![0, 1, 2],
        &mini_gpu.renderer,
    );
    let material = Material::new(
        MaterialConfig {
            shader_text: include_str!("./triangle.wgsl").to_string(),
            topology: wgpu::PrimitiveTopology::TriangleList,
            uniforms: vec![1., 1., 0., 0.5],
        },
        &mini_gpu.renderer,
    );

    let material_line = Material::new(
        MaterialConfig {
            shader_text: include_str!("./triangle.wgsl").to_string(),
            topology: wgpu::PrimitiveTopology::LineStrip,
            uniforms: vec![1., 0., 0.5, 1.],
        },
        &mini_gpu.renderer,
    );
    //object1
    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    let mesh_index = mini_gpu
        .scene
        .set_entity_component::<Mesh>(entity_id, mesh, "mesh");

    let material_index = mini_gpu
        .scene
        .set_entity_component::<Material>(entity_id, material, "material");

    //object2
    let entity_2 = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu
        .scene
        .set_entity_component_index(entity_2, mesh_index, "mesh");
    let material_line_index =
        mini_gpu
            .scene
            .set_entity_component::<Material>(entity_2, material_line, "material");

    //object3
    let mesh_2 = Mesh::new(
        vec![-1., -1., 1., 1., 1., -1.],
        vec![0, 1, 2],
        &mini_gpu.renderer,
    );
    let entity_3 = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu
        .scene
        .set_entity_component::<Mesh>(entity_3, mesh_2, "mesh");
    mini_gpu
        .scene
        .set_entity_component_index(entity_3, material_line_index, "material");
}
