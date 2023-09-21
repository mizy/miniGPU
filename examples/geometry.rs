use ::mini_gpu::{
    components::{
        controller::map::MapController,
        lights::directional_light::{DirectionalLight, DirectionalLightUniform},
        lights::light::LightRef,
        material::{Material, MaterialConfig, MaterialRef},
        mesh::Mesh,
    },
    entity::Entity,
    geometry::sphere,
    light_ref, material_ref,
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
};
use mini_gpu::mini_gpu::MiniGPUConfig;
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
        let camera = mini_gpu.scene.get_camera_mut().unwrap();
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
                        mini_gpu.scene.get_camera_mut().unwrap().set_aspect(
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
    let mesh = sphere::make_sphere_mesh(
        sphere::MakeSphereConfig {
            width_segments: 64,
            height_segments: 64,
            ..Default::default()
        },
        &mini_gpu.renderer,
    );
    let material = material_ref!(Material::new(
        MaterialConfig {
            shader: include_str!("./geometry.wgsl").to_string(),
            topology: wgpu::PrimitiveTopology::TriangleList,
            uniforms: vec![0., 0.2, 0.5, 0.4],
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
        .set_entity_component(entity_id, material, "material");

    // add light
    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    let light = DirectionalLight::new(
        &mini_gpu.renderer,
        1,
        DirectionalLightUniform {
            intensity: 1.,
            direction: [1., 1., -1.],
            color: [1., 1., 0.8],
            _padding: 0.,
        },
    );
    mini_gpu
        .scene
        .set_entity_component(entity_id, light_ref!(light), "light");
}
