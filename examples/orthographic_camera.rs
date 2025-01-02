use ::mini_gpu::{
    components::{
        controller::map::MapController,
        material::MaterialTrait,
        materials::image::{Image, ImageConfig},
        orthographic_camera::OrthographicCamera,
    },
    entity::Entity,
    mini_gpu::{self, MiniGPU},
    system::mesh_render::MeshRender,
    utils::{self, test_xyz, texture::Texture},
};
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
    utils::camera::default_orthographic_camera(&mut mini_gpu);
    make_test_mesh(&mut mini_gpu).await;
    test_xyz::add_xyz_line(&mut mini_gpu, Some(10.));
    let mut camera_controller = MapController::default();
    camera_controller.config.width = mini_gpu.renderer.viewport.width;
    camera_controller.config.height = mini_gpu.renderer.viewport.height;

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
                            camera_controller.config.width = mini_gpu.renderer.viewport.width;
                            camera_controller.config.height = mini_gpu.renderer.viewport.height;
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

    let camera = mini_gpu.scene.get_default_camera().unwrap();
    let orthographic_camera = camera
        .as_any()
        .downcast_mut::<OrthographicCamera>()
        .unwrap();
    orthographic_camera.config.width = 10.0;
    orthographic_camera.config.aspect = mini_gpu.renderer.viewport.aspect;
    orthographic_camera.config.position.z = 10.0;
    camera.update_bind_group(&mini_gpu.renderer);
}
