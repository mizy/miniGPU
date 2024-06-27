use components::{controller::map::MapController, materials::sprite::SpriteMaterialConfig};
use entity::{sprite_entity, Entity};
use image::{ImageBuffer, Rgba};
use ::mini_gpu::{
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
    utils::test_xyz,
    *,
};
use utils::texture;
use wasm_bindgen::prelude::*;
use winit::{event::{Event, WindowEvent}, event_loop::{EventLoop}};

#[wasm_bindgen]
pub struct MiniGPUWeb {
    mini_gpu_instance: MiniGPU,
    camera_controller: MapController,
    now_window_id: winit::window::WindowId,
    event_loop: Option<EventLoop<()>>,
}
#[wasm_bindgen]
impl MiniGPUWeb {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> MiniGPUWeb {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        log::info!("init wasm example");
        let (event_loop, mut mini_gpu_instance,now_window_id) = MiniGPUWeb::init_instance().await;
        let camera_controller = MapController::default();
        make_test_mesh(&mut mini_gpu_instance);
        MiniGPUWeb {
            mini_gpu_instance,
            event_loop: Some(event_loop),
            now_window_id,
            camera_controller,
        }
    }

    async fn init_instance()->(EventLoop<()>,MiniGPU,winit::window::WindowId){ 
        let event_loop = EventLoop::new().unwrap();
        let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            use web_sys::HtmlCanvasElement;
            // let _ = window.request_inner_size(PhysicalSize::new(1000, 800));
            
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = window.canvas().unwrap();
                    let canvas_ele = web_sys::Element::from(canvas);
                    let canvas_html: HtmlCanvasElement = canvas_ele.dyn_into().unwrap();
                    canvas_html.set_width(800);
                    canvas_html.set_height(600);
                    dst.append_child(&canvas_html).ok()?;
                    Some(())
                })
                .expect("无法将画布添加到网页上");
        }

        let now_window_id = window.id();

        let mut mini_gpu_instance = mini_gpu::MiniGPU::new(
            mini_gpu::MiniGPUConfig {
                width: 800,
                height: 600,
            },
            window,
        )
        .await;
        mini_gpu_instance
            .renderer
            .add_system("render".to_string(), Box::new(MeshRender {}));
            // make_test_mesh(&mut mini_gpu);
            // test_xyz::add_xyz_line(&mut mini_gpu_instance);

        (event_loop,  mini_gpu_instance,now_window_id)
    }

    #[wasm_bindgen]
    pub async fn loop_render(&mut self){
        let camera_controller = &mut self.camera_controller;
        let now_window_id = self.now_window_id;
        let mini_gpu_instance = &mut self.mini_gpu_instance;
        let now_event_loop = self.event_loop.take().unwrap();
        now_event_loop.run(move |event, target| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == now_window_id => {
               camera_controller.process_events(event);
                match event {
                    WindowEvent::RedrawRequested => {
                        let camera = mini_gpu_instance.scene.get_default_camera().unwrap();
                        camera_controller.update(camera);
                        camera.update_bind_group(&mini_gpu_instance.renderer);
                        if let Err(e) = mini_gpu_instance.renderer.render(&mini_gpu_instance.scene) {
                            println!("Failed to render: {}", e);
                        }
                    }
                    // WindowEvent::Resized(physical_size) => {
                    //     let camera = mini_gpu.scene.get_default_camera().unwrap();
                    //     mini_gpu
                    //         .renderer
                    //         .resize(physical_size.width, physical_size.height);
                    //     camera.set_aspect(
                    //         physical_size.width as f32 / physical_size.height as f32,
                    //         &mini_gpu.renderer,
                    //     );
                    //     mini_gpu.renderer.window.request_redraw();
                    // }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                }
            }
            Event::AboutToWait => {
                mini_gpu_instance.renderer.window.request_redraw();
            }
            _ => {}
        })
        .unwrap(); 
    }

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
    entity::mesh_line::make_mesh(
        &vec![
            glam::Vec3::new(0.0, 0.0, 0.0),
            glam::Vec3::new(0.5, 0.0, 0.0),
        ],
        60.0 as f32 / mini_gpu.config.width as f32,
        &mini_gpu.renderer,
        &mut mini_gpu.scene,
        entity_line_id,
    );
    entity::mesh_line::make_material(
        &mini_gpu.renderer,
        &mut mini_gpu.scene,
        vec![1.0, 1.0, 1.0, 1.0],
        entity_line_id,
    );
}
