use std::f64::consts::PI;

use ::mini_gpu::{
    components::{
        controller::map::MapController,
        lights::directional_light::{DirectionalLight, DirectionalLightUniform},
        mesh::Mesh,
    },
    entity::Entity,
    geometry::sphere,
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
};
use mini_gpu::{
    components::{
        lights::light::LightTrait,
        material::MaterialTrait,
        materials::image::{Image, ImageConfig},
    },
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

    // add light
    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    let light = DirectionalLight::new(
        &mini_gpu.renderer,
        1,
        DirectionalLightUniform {
            intensity: 1.,
            direction: [1., 1., -1.],
            color: [1., 1., 0.8, 1.0],
        },
    );
    let light_index = mini_gpu.scene.set_entity_component::<Box<dyn LightTrait>>(
        entity_id,
        Box::new(light),
        "light",
    );

    let mut camera_controller = MapController::default();

    mini_gpu
        .renderer
        .add_system("render".to_string(), Box::new(MeshRender {}));
    event_loop
        .run(move |event, target| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == mini_gpu.renderer.window.id() => {
                camera_controller.process_events(event);
                match event {
                    WindowEvent::RedrawRequested => {
                        let camera = mini_gpu.scene.get_default_camera().unwrap();
                        update_light(&mini_gpu, light_index);
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

fn update_light(mini_gpu: &MiniGPU, light_index: usize) {
    // update light position
    let light = mini_gpu
        .scene
        .get_component::<Box<DirectionalLight>>(light_index);
    let now_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let delta = (now_time as f64 / 1000.) % (2. * PI);
    light.uniform.direction[0] = f32::sin(delta as f32);
    light.uniform.direction[2] = 1.0 * f32::cos(delta as f32);
    light.update_buffer(&mini_gpu.renderer);
}

fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    let mesh = sphere::make_sphere_mesh(
        sphere::MakeSphereConfig {
            width_segments: 64,
            height_segments: 64,
            ..Default::default()
        },
        &mini_gpu.renderer,
    );

    let texture = texture::Texture::from_bytes(
        &mini_gpu.renderer.device,
        &mini_gpu.renderer.queue,
        include_bytes!("./case.jpg"),
        "texture.jpg",
    )
    .unwrap();
    let material = Box::new(Image::new(
        ImageConfig {
            shader: Some(include_str!("./geometry.wgsl").to_string()),
            texture: Some(texture),
            ..Default::default()
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
