use ::mini_gpu::{
    components::{
        material::MaterialTrait,
        materials::image::{Image, ImageConfig},
        perspective_camera::PerspectiveCamera,
    },
    entity::Entity,
    mini_gpu,
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
    utils::texture::Texture,
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
                } if window_id == window.id() => match event {
                    WindowEvent::RedrawRequested => {
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
                },
                Event::AboutToWait => {
                    mini_gpu.renderer.window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
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
    let _ = mini_gpu
        .renderer
        .window
        .request_inner_size(LogicalSize::new(image.width(), image.height()));
}
