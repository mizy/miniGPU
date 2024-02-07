use ::mini_gpu::{
    components::controller::map::MapController,
    entity::{sprite_entity, Entity},
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
};
use image::{ImageBuffer, Rgba};
use mini_gpu::{
    components::{
        material::{Material, MaterialConfig, MaterialTrait},
        materials::sprite::SpriteMaterialConfig,
    },
    mini_gpu::MiniGPUConfig,
    utils::{test_xyz, texture},
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
    let mut mini_gpu = MiniGPU::new(
        MiniGPUConfig {
            width: size.width,
            height: size.height,
        },
        window,
    )
    .await;
    make_test_mesh(&mut mini_gpu);
    mini_gpu::utils::camera::default_orthographic_camera(&mut mini_gpu);
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

fn create_solid_color_image(width: u32, height: u32, color: [u8; 4]) -> image::DynamicImage {
    let img_buffer = ImageBuffer::from_fn(width, height, |_, _| Rgba(color));
    image::DynamicImage::ImageRgba8(img_buffer)
}

fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    // add point
    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    sprite_entity::make_mesh(
        glam::Vec3::new(0.0, 0.0, 0.0),
        &mini_gpu.renderer,
        &mut mini_gpu.scene,
        entity_id,
    );
    let img = create_solid_color_image(256, 256, [255, 0, 0, 255]);
    let texture = texture::Texture::from_image(
        &mini_gpu.renderer.device,
        &mini_gpu.renderer.queue,
        &img,
        Some("test"),
    )
    .unwrap();
    sprite_entity::make_material(
        &mini_gpu.renderer,
        &mut mini_gpu.scene,
        SpriteMaterialConfig {
            width: texture.size.width as f32 / mini_gpu.config.width as f32,
            height: texture.size.height as f32 / mini_gpu.config.width as f32,
            radial: true,
            texture: Some(texture),
            ..Default::default()
        },
        entity_id,
    );
    let material_id = &mini_gpu
        .scene
        .get_entity_component_index(entity_id, "material");

    let entity_id2 = mini_gpu.scene.add_entity(Entity::new());
    sprite_entity::make_mesh(
        glam::Vec3::new(0.5, 0.0, 0.0),
        &mini_gpu.renderer,
        &mut mini_gpu.scene,
        entity_id2,
    );
    mini_gpu
        .scene
        .set_entity_component_index(entity_id2, *material_id, "material");
    //add line
    let entity_line_id = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu::entity::mesh_line::make_mesh(
        &vec![
            glam::Vec3::new(0.0, 0.0, 0.0),
            glam::Vec3::new(0.5, 0.0, 0.0),
        ],
        60.0 as f32 / mini_gpu.config.width as f32,
        &mini_gpu.renderer,
        &mut mini_gpu.scene,
        entity_line_id,
    );
    mini_gpu::entity::mesh_line::make_material(
        &mini_gpu.renderer,
        &mut mini_gpu.scene,
        vec![1.0, 1.0, 1.0, 1.0],
        entity_line_id,
    );
}
