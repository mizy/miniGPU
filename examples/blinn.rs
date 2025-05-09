use ::mini_gpu::{
    components::{
        controller::map::MapController,
        material::{Material, MaterialConfig, MaterialTrait},
        materials::{
            basic::{BasicMaterial, BasicMaterialConfig},
            blinn_phong::{BlinnPhongMaterial, BlinnPhongMaterialConfig},
        },
        mesh::{Mesh, VertexFormat, VertexPositionNormal},
        perspective_camera::PerspectiveCamera,
    },
    entity,
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
    utils::{self},
};
use glam::Vec3;
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
    let vertices: Vec<VertexPositionNormal> = vec![
        VertexPositionNormal {
            position: [0.5, 0.5, 0.],
            normal: [0., 0., 1.],
        },
        VertexPositionNormal {
            position: [-0.5, 0.5, 0.],
            normal: [0., 0., 1.],
        },
        VertexPositionNormal {
            position: [0., 0., 0.],
            normal: [0., 0., 1.],
        },
    ];

    let mesh = Mesh::new(
        bytemuck::cast_slice(&vertices),
        vec![0, 1, 2],
        VertexFormat::PositionNormal,
        &mini_gpu.renderer,
    );

    let mut default_material_config = BlinnPhongMaterialConfig::default();
    default_material_config.diffuse_color = Vec3::new(1.0, 0.0, 0.0);
    let material = BlinnPhongMaterial::new(default_material_config, &mini_gpu.renderer);

    // let material = BasicMaterial::new(
    //     BasicMaterialConfig {
    //         name: "basic_material".to_string(),
    //         shader: None,
    //         texture: None,
    //         color: [1.0, 0.0, 0.0, 1.0],
    //     },
    //     &mini_gpu.renderer,
    // );

    let entity = entity::Entity::new();
    let entity_id = mini_gpu.scene.add_entity(entity);
    mini_gpu
        .scene
        .set_entity_component::<Mesh>(entity_id, mesh, "mesh");
    mini_gpu
        .scene
        .set_entity_component::<Box<dyn MaterialTrait>>(entity_id, Box::new(material), "material");

    let camera = mini_gpu.scene.get_default_camera().unwrap();
    let perspective_camera = camera.as_any().downcast_mut::<PerspectiveCamera>().unwrap();
    perspective_camera.config.position.z = 10.0;
    camera.update_bind_group(&mini_gpu.renderer);
}
