use std::f32::consts::PI;

use ::mini_gpu::{
    components::{
        controller::map::MapController,
        instance::{self, Instance},
        material::MaterialRef,
        materials::image::{Image, ImageConfig},
    },
    entity::Entity,
    material_ref, mini_gpu,
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
};
use bytemuck::{Pod, Zeroable};
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
    let mut camera_controller = MapController::default();

    mini_gpu
        .renderer
        .add_system("render".to_string(), Box::new(MeshRender {}));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let window = &mini_gpu.renderer.window;

        match event {
            Event::RedrawRequested(_) => {
                let camera = mini_gpu.scene.get_camera_mut().unwrap();
                camera_controller.update(camera);
                camera.update_bind_group(&mini_gpu.renderer);
                if let Err(e) = mini_gpu.renderer.render(&mut mini_gpu.scene) {
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
}

fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    let image = image::load_from_memory(include_bytes!("./case.jpg")).unwrap();
    let material = Image::new(
        ImageConfig {
            width: image.width(),
            height: image.height(),
            diffuse_data: image.to_rgba8().into_raw(),
            shader: Some(include_str!("./instance.wgsl").to_string()),
            ..Default::default()
        },
        &mini_gpu.renderer,
    );
    println!("width: {}", image.width());
    println!("height: {}", image.height());
    let scale = image.width() as f32 / image.height() as f32;
    let mesh = material.make_image_mesh(scale * 1., 1., &mini_gpu.renderer);

    let camera = mini_gpu.scene.get_camera_mut().unwrap();
    camera.config.position = glam::Vec3::new(0., 0., 2.);
    camera.update_bind_group(&mini_gpu.renderer);

    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu.scene.set_entity_component(entity_id, mesh, "mesh");
    mini_gpu
        .scene
        .set_entity_component(entity_id, material_ref!(material), "material");
    mini_gpu
        .renderer
        .window
        .set_inner_size(LogicalSize::new(image.width(), image.height()));
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
