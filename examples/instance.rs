use std::f32::consts::PI;

use ::mini_gpu::{
    components::{
        controller::map::MapController,
        instance::{self, Instance},
        material::MaterialTrait,
        materials::basic::{BasicMaterial, BasicMaterialConfig},
        perspective_camera::PerspectiveCamera,
    },
    entity::Entity,
    geometry::plane::{make_plane_mesh, MakePlaneConfig},
    mini_gpu::{self, MiniGPU},
    system::mesh_render::MeshRender,
    utils::texture::Texture,
};
use winit::{
    dpi::LogicalSize,
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
    make_test_mesh(&mut mini_gpu);
    let mut camera_controller = MapController::default();
    camera_controller.config.scale_factor = mini_gpu.renderer.viewport.scale_factor;

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
                            if let Err(e) = mini_gpu.renderer.render(&mut mini_gpu.scene) {
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
    let image = image::load_from_memory(include_bytes!("./case.jpg")).unwrap();
    let texture = Texture::from_image(
        &mini_gpu.renderer.device,
        &mini_gpu.renderer.queue,
        &image,
        Some("image"),
    )
    .unwrap();
    let material = BasicMaterial::new(
        BasicMaterialConfig {
            texture: Some(texture),
            shader: Some(include_str!("./instance.wgsl").to_string()),
            ..Default::default()
        },
        &mini_gpu.renderer,
    );
    println!("width: {}", image.width());
    println!("height: {}", image.height());
    let scale = image.width() as f32 / image.height() as f32;
    let mesh = make_plane_mesh(
        MakePlaneConfig {
            width: 1.0 * scale,
            height: 1.0,
            ..Default::default()
        },
        &mini_gpu.renderer,
    );

    let camera = mini_gpu.scene.get_default_camera().unwrap();
    let perspective_camera = camera.as_any().downcast_mut::<PerspectiveCamera>().unwrap();
    perspective_camera.config.position = glam::Vec3::new(0., 0., 2.);
    camera.update_bind_group(&mini_gpu.renderer);

    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu.scene.set_entity_component(entity_id, mesh, "mesh");
    mini_gpu
        .scene
        .set_entity_component::<Box<dyn MaterialTrait>>(entity_id, Box::new(material), "material");
    let size_result = mini_gpu
        .renderer
        .window
        .request_inner_size(LogicalSize::new(image.width(), image.height()));
    println!("resize: {:?}", size_result);
    //instance
    let mut instance_data = Vec::new();
    for i in 0..10 {
        for j in 0..10 {
            let matdata = glam::Mat4::from_scale_rotation_translation(
                glam::vec3(
                    (i + 1) as f32 / 10.,
                    (i + 1) as f32 / 10.,
                    (i + 1) as f32 / 10.,
                ),
                glam::Quat::from_euler(glam::EulerRot::XYZ, i as f32 / 10. * PI, 0., 0.),
                glam::vec3(i as f32, j as f32, 0.0),
            );
            instance_data.push(instance::InstanceData {
                data: matdata.to_cols_array_2d(),
            });
        }
    }
    let instance = Instance::new(instance_data, &mini_gpu.renderer);
    mini_gpu
        .scene
        .set_entity_component(entity_id, instance, "instance");
}
