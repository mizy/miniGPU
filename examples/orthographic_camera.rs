use ::mini_gpu::{
    components::{
        controller::map::MapController,
        material::MaterialTrait,
        materials::image::{Image, ImageConfig},
        mesh::Mesh,
        orthographic_camera::{self, OrthographicCamera, OrthographicCameraConfig},
        perspective_camera::{CameraTrait, PerspectiveCamera},
    },
    entity::{self, Entity},
    geometry::sphere,
    mini_gpu,
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
    utils::{
        self,
        texture::{self, Texture},
    },
};
use winit::{
    dpi::LogicalSize,
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
                        camera.set_aspect(
                            physical_size.width as f32 / physical_size.height as f32,
                            &mini_gpu.renderer,
                        );
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
    let bytes = include_bytes!("./case.jpg");
    let image = image::load_from_memory(bytes).unwrap();
    let texture = Texture::from_image(
        &mini_gpu.renderer.device,
        &mini_gpu.renderer.queue,
        &image,
        Some("texture"),
    )
    .unwrap();

    let material = Image::new(
        ImageConfig {
            texture: Some(texture),
            ..Default::default()
        },
        &mini_gpu.renderer,
    );
    println!("width: {}", image.width());
    println!("height: {}", image.height());
    let scale = image.width() as f32 / image.height() as f32;
    let mesh = material.make_image_mesh(scale * 1., 1., &mini_gpu.renderer);

    let camera = mini_gpu.scene.get_default_camera().unwrap();
    let perspective_camera = camera.as_any().downcast_mut::<PerspectiveCamera>().unwrap();
    perspective_camera.config.position = glam::Vec3::new(0., 0., 2.);
    camera.update_bind_group(&mini_gpu.renderer);

    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu.scene.set_entity_component(entity_id, mesh, "mesh");
    mini_gpu
        .scene
        .set_entity_component::<Box<dyn MaterialTrait>>(entity_id, Box::new(material), "material");
    mini_gpu
        .renderer
        .window
        .set_inner_size(LogicalSize::new(image.width(), image.height()));
}

fn add_camera(mini_gpu: &mut MiniGPU) {
    let mut camera = OrthographicCamera::new(
        OrthographicCameraConfig {
            ..Default::default()
        },
        &mini_gpu.renderer,
    );
    camera.config.position.z = 10.0;
    let entity_id = mini_gpu.scene.add_entity(entity::Entity::new());
    mini_gpu.scene.set_entity_component::<Box<dyn CameraTrait>>(
        entity_id,
        Box::new(camera),
        "camera",
    );
    mini_gpu.scene.default_camera = Some(entity_id);
}
