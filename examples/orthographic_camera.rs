use ::mini_gpu::{
    components::{
        controller::map::MapController,
        orthographic_camera::{self, OrthographicCamera, OrthographicCameraConfig},
        perspective_camera::{CameraTrait, PerspectiveCamera},
    },
    entity, mini_gpu,
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
    utils,
};
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
    make_test_mesh(&mut mini_gpu).await;
    add_camera(&mut mini_gpu);
    let mut camera_controller = MapController::default();

    mini_gpu
        .renderer
        .add_system("render".to_string(), Box::new(MeshRender {}));
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let window = &mini_gpu.renderer.window;
        let camera = mini_gpu.scene.get_default_camera().unwrap();
        match event {
            Event::RedrawRequested(_) => {
                camera_controller.update(camera);
                camera.update_bind_group(&mini_gpu.renderer);
                if let Err(e) = mini_gpu.renderer.render(&mini_gpu.scene) {
                    println!("Failed to render: {}", e);
                }
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                camera_controller.process_events(event);
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

async fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    let path = std::path::Path::new("examples/models/cube/cube.obj");
    let obj = utils::obj::load_obj(path, mini_gpu).await;
    match obj {
        Ok(size) => {
            println!("Loaded obj with {} vertices", size);
        }
        Err(e) => {
            println!("Failed to load obj ({:?})", e,);
        }
    }
}

fn add_camera(mini_gpu: &mut MiniGPU) {
    let camera = OrthographicCamera::new(
        OrthographicCameraConfig {
            ..Default::default()
        },
        &mini_gpu.renderer,
    );
    let entity_id = mini_gpu.scene.add_entity(entity::Entity::new());
    mini_gpu.scene.set_entity_component::<Box<dyn CameraTrait>>(
        entity_id,
        Box::new(camera),
        "camera",
    );
    mini_gpu.scene.default_camera = Some(entity_id);
}
