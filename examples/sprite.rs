use ::mini_gpu::{
    components::controller::orbit::OrbitController,
    entity::{sprite_entity, Entity},
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
};
use mini_gpu::{
    components::materials::sprite::SpriteMaterialConfig, mini_gpu::MiniGPUConfig, utils::texture,
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
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
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
    let mut camera_controller = OrbitController::default();

    mini_gpu
        .renderer
        .add_system("render".to_string(), Box::new(MeshRender {}));
    event_loop
        .run(move |event, target| {
            let window = &mini_gpu.renderer.window;
            let camera = mini_gpu.world.get_default_camera().unwrap();

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
                            if let Err(e) = mini_gpu.renderer.render(&mini_gpu.world) {
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

fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    let entity_id = mini_gpu.world.add_entity(Entity::new());
    sprite_entity::make_mesh(
        glam::Vec3::new(0.0, 0.0, 0.0),
        &mini_gpu.renderer,
        &mut mini_gpu.world,
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
        &mut mini_gpu.world,
        SpriteMaterialConfig {
            width: texture.size.width as f32 / mini_gpu.config.width as f32,
            height: texture.size.height as f32 / mini_gpu.config.width as f32,
            radial: true,
            texture: Some(texture),
            ..Default::default()
        },
        entity_id,
    )
}
