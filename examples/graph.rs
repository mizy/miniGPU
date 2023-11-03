use ::mini_gpu::{
    components::controller::map::MapController,
    entity::{sprite_entity, Entity},
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
};
use mini_gpu::{
    components::{materials::sprite::SpriteMaterialConfig, perspective_camera::PerspectiveCamera},
    mini_gpu::MiniGPUConfig,
    utils::texture,
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
    let mut mini_gpu = MiniGPU::new(
        MiniGPUConfig {
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
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let window = &mini_gpu.renderer.window;
        let camera = mini_gpu.scene.get_default_camera().unwrap();
        let perspective_camera = camera.as_any().downcast_mut::<PerspectiveCamera>().unwrap();
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
                        perspective_camera.set_aspect(
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
    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    sprite_entity::make_mesh(
        glam::Vec3::new(0.0, 0.0, 0.0),
        &mini_gpu.renderer,
        &mut mini_gpu.scene,
        entity_id,
    );
    let texture = texture::Texture::from_bytes(
        &mini_gpu.renderer.device,
        &mini_gpu.renderer.queue,
        include_bytes!("./case.jpg"),
        "case.jpg",
    )
    .unwrap();
    sprite_entity::make_material(
        &mini_gpu.renderer,
        &mut mini_gpu.scene,
        SpriteMaterialConfig {
            width: texture.size.width as f32 / mini_gpu.config.width as f32,
            height: texture.size.height as f32 / mini_gpu.config.height as f32,
            radial: true,
            texture: Some(texture),
            ..Default::default()
        },
        entity_id,
    )
}
