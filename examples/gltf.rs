use std::fs;

use ::mini_gpu::{
    components::{
        controller::map::MapController,
        material::{Material, MaterialConfig, MaterialTrait},
        mesh::Mesh,
        perspective_camera::PerspectiveCamera,
    },
    entity::Entity,
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
    utils::{self},
};
use mini_gpu::mini_gpu;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
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
    make_test_mesh(&mut mini_gpu).await;
    let mut camera_controller = MapController::default();

    mini_gpu
        .renderer
        .add_system("render".to_string(), Box::new(MeshRender {}));
    event_loop
        .run(move |event, target| {
            let window = &mini_gpu.renderer.window;
            let camera = mini_gpu.scene.get_default_camera().unwrap();
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    camera_controller.process_events(event);
                    match event {
                        WindowEvent::RedrawRequested => {
                            camera_controller.update(camera);
                            camera.update_bind_group(&mini_gpu.renderer);
                            if let Err(e) = mini_gpu.renderer.render(&mini_gpu.scene) {
                                println!("Failed to render: {}", e);
                            }
                        }
                        WindowEvent::Resized(physical_size) => {
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
            }
        })
        .unwrap();
}

async fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    let path = std::path::Path::new("examples/models/SheenChair.glb");
    let buffer = std::fs::read(path).unwrap();
    let obj = utils::gltf::load_gltf(&buffer, mini_gpu).await;
    match obj {
        Ok(size) => {
            println!("Loaded obj with entity {} ", size);
        }
        Err(e) => {
            println!("Failed to load obj ({:?})", e,);
        }
    }

    let camera = mini_gpu.scene.get_default_camera().unwrap();
    let perspective_camera = camera.as_any().downcast_mut::<PerspectiveCamera>().unwrap();
    perspective_camera.config.position.z = 10.0;
    camera.update_bind_group(&mini_gpu.renderer);
}
