use ::mini_gpu::{
    components::controller::map::MapController, system::mesh_render::MeshRender, utils::axis,
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
    axis::add_xyz_line(&mut mini_gpu, None);
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
