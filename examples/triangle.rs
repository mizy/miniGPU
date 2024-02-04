use ::mini_gpu::{
    components::{
        material::{Material, MaterialConfig, MaterialTrait},
        mesh::Mesh,
    },
    entity::Entity,
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
    let event_loop = EventLoop::new().unwrap();
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
        .renderer
        .add_system("render".to_string(), Box::new(MeshRender {}));

    event_loop
        .run(move |event, target| {
            let window = &mini_gpu.renderer.window;
            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    match event {
                        WindowEvent::RedrawRequested => {
                            if let Err(e) = mini_gpu.renderer.render(&mini_gpu.scene) {
                                println!("Failed to render: {}", e);
                            }
                        }
                        WindowEvent::Resized(physical_size) => {
                            mini_gpu
                                .renderer
                                .resize(physical_size.width, physical_size.height);
                            mini_gpu.renderer.window.request_redraw();
                        }

                        WindowEvent::CloseRequested => target.exit(),
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    mini_gpu.renderer.window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}

fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    let mesh = Mesh::new(
        vec![0.5, 0.5, 0., -0.5, 0.5, 0., 0., 0., 0.],
        vec![0, 1, 2],
        &mini_gpu.renderer,
    );
    let material_line = Material::new(
        MaterialConfig {
            shader: include_str!("./triangle.wgsl").to_string(),
            topology: wgpu::PrimitiveTopology::TriangleList,
            uniforms: vec![1., 0., 0.5, 1.],
        },
        &mini_gpu.renderer,
    );
    //object1
    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu
        .scene
        .set_entity_component::<Mesh>(entity_id, mesh, "mesh");
    let material_index = mini_gpu
        .scene
        .set_entity_component::<Box<dyn MaterialTrait>>(
            entity_id,
            Box::new(material_line),
            "material",
        );

    //object3 test reuse material1
    let mesh_2 = Mesh::new(
        vec![-1., -1., 1., 1., 1., 0., 1., -1., 1.],
        vec![0, 1, 2],
        &mini_gpu.renderer,
    );
    let entity_3 = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu
        .scene
        .set_entity_component::<Mesh>(entity_3, mesh_2, "mesh");
    mini_gpu
        .scene
        .set_entity_component_index(entity_3, material_index, "material");
}
