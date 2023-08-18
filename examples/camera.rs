use ::mini_gpu::{
    components::{
        material::{Material, MaterialConfig, MaterialRef, MaterialTrait},
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
        vec![2., 1., -1., 1., 0., 0.],
        vec![0, 1, 2],
        &mini_gpu.renderer,
    );
    let material = new_material(Material::new(
        MaterialConfig {
            shader_text: include_str!("./camera.wgsl").to_string(),
            topology: wgpu::PrimitiveTopology::TriangleList,
            uniforms: vec![0., 0.2, 0.5, 0.4],
        },
        &mini_gpu.renderer,
    ));
    let camera = mini_gpu.scene.get_camera_mut().unwrap();
    camera.config.position = glam::Vec3::new(0., 0., 2.);
    camera.update_bind_group(&mini_gpu.renderer);
    let m = material.get_material::<Material>();
    println!("m's name is {}", m.get_name());
    //object1
    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu
        .scene
        .set_entity_component::<Mesh>(entity_id, mesh, "mesh");
    mini_gpu
        .scene
        .set_entity_component(entity_id, material, "material");
}

fn new_material(m: Material) -> MaterialRef {
    MaterialRef::new(Box::new(m))
}
