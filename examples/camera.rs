use ::mini_gpu::{
    components::{
        controller::map::MapController,
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
    let now_window_id = window.id();
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
    let mut camera_controller = MapController::default();
    mini_gpu
        .renderer
        .add_system("render".to_string(), Box::new(MeshRender {}));
    event_loop
        .run(move |event, target| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == now_window_id => {
                camera_controller.process_events(event);
                match event {
                    WindowEvent::RedrawRequested => {
                        let camera = mini_gpu.scene.get_default_camera().unwrap();
                        println!("{:?}", std::time::Instant::now().elapsed().as_millis());
                        camera_controller.update(camera);
                        camera.update_bind_group(&mini_gpu.renderer);
                        if let Err(e) = mini_gpu.renderer.render(&mini_gpu.scene) {
                            println!("Failed to render: {}", e);
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        let camera = mini_gpu.scene.get_default_camera().unwrap();
                        mini_gpu
                            .renderer
                            .resize(physical_size.width, physical_size.height);
                        camera.set_aspect(
                            physical_size.width as f32 / physical_size.height as f32,
                            &mini_gpu.renderer,
                        );
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
        })
        .unwrap();
}

fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    let mesh = Mesh::new(
        vec![0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5],
        vec![0, 1, 0, 2, 0, 3],
        &mini_gpu.renderer,
    );
    let material = Box::new(Material::new(
        MaterialConfig {
            shader: include_str!("./camera.wgsl").to_string(),
            topology: wgpu::PrimitiveTopology::LineList,
            uniforms: vec![0., 0.2, 0.5, 0.4],
        },
        &mini_gpu.renderer,
    ));
    //object1
    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu
        .scene
        .set_entity_component::<Mesh>(entity_id, mesh, "mesh");
    mini_gpu
        .scene
        .set_entity_component::<Box<dyn MaterialTrait>>(entity_id, material, "material");
}
