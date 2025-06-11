use std::collections::HashMap;

use ::mini_gpu::{mini_gpu::MiniGPU, system::mesh_render::MeshRender, *};
use components::{
    controller::orbit::OrbitController, materials::sprite::SpriteMaterialConfig,
    perspective_camera::PerspectiveCamera,
};
use entity::{sprite_entity, Entity};
use image::{ImageBuffer, Rgba};
use utils::texture;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Math::random;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

#[wasm_bindgen]
pub struct MiniGPUWeb {
    mini_gpu_instance: MiniGPU,
    camera_controller: OrbitController,
    now_window_id: winit::window::WindowId,
    event_loop: Option<EventLoop<()>>,
    assets_buffer_map: HashMap<String, Vec<u8>>,
}

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
#[wasm_bindgen]
impl MiniGPUWeb {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> MiniGPUWeb {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        log::info!("init wasm example");
        let (event_loop, mut mini_gpu_instance, now_window_id) = MiniGPUWeb::init_instance().await;
        let camera_controller = OrbitController::default();
        make_test_mesh(&mut mini_gpu_instance);
        make_obj_mesh(&mut mini_gpu_instance).await;
        MiniGPUWeb {
            mini_gpu_instance,
            event_loop: Some(event_loop),
            now_window_id,
            camera_controller,
            assets_buffer_map: HashMap::new(),
        }
    }

    async fn init_instance() -> (EventLoop<()>, MiniGPU, winit::window::WindowId) {
        let event_loop = EventLoop::new().unwrap();
        let window = winit::window::WindowBuilder::new()
            .build(&event_loop)
            .unwrap();
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::HtmlCanvasElement;
            use winit::platform::web::WindowExtWebSys;
            // let _ = window.request_inner_size(PhysicalSize::new(1000, 800));

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = window.canvas().unwrap();
                    let canvas_ele = web_sys::Element::from(canvas);
                    let canvas_html: HtmlCanvasElement = canvas_ele.dyn_into().unwrap();
                    let style = canvas_html.style();
                    style
                        .set_property("width", &format!("{}px", WIDTH))
                        .expect("设置宽度失败");
                    style
                        .set_property("height", &format!("{}px", HEIGHT))
                        .expect("设置高度失败");
                    dst.append_child(&canvas_html).ok()?;
                    Some(())
                })
                .expect("无法将画布添加到网页上");
        }

        let now_window_id = window.id();

        let mut mini_gpu_instance = mini_gpu::MiniGPU::new(
            mini_gpu::MiniGPUConfig {
                width: WIDTH * 2,
                height: HEIGHT * 2,
            },
            window,
        )
        .await;
        mini_gpu_instance
            .renderer
            .add_system("render".to_string(), Box::new(MeshRender {}));

        (event_loop, mini_gpu_instance, now_window_id)
    }

    #[wasm_bindgen]
    pub async fn loop_render(&mut self) {
        let camera_controller = &mut self.camera_controller;
        let now_window_id = self.now_window_id;
        let mini_gpu_instance = &mut self.mini_gpu_instance;
        let now_event_loop = self.event_loop.take().unwrap();
        now_event_loop
            .run(move |event, target| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == now_window_id => {
                    camera_controller.process_events(event);
                    match event {
                        WindowEvent::RedrawRequested => {
                            let camera = mini_gpu_instance.world.get_default_camera().unwrap();
                            camera_controller.update(camera);
                            camera.update_bind_group(&mini_gpu_instance.renderer);
                            if let Err(e) =
                                mini_gpu_instance.renderer.render(&mini_gpu_instance.world)
                            {
                                println!("Failed to render: {}", e);
                            }
                        }
                        // WindowEvent::Resized(physical_size) => {
                        //     let camera = mini_gpu.world.get_default_camera().unwrap();
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

    #[wasm_bindgen]
    pub fn update_obj_map(&mut self, key: String, value: Vec<u8>) {
        self.assets_buffer_map.insert(key, value);
    }
}

fn create_solid_color_image(width: u32, height: u32, color: [u8; 4]) -> image::DynamicImage {
    let img_buffer = ImageBuffer::from_fn(width, height, |_, _| Rgba(color));
    image::DynamicImage::ImageRgba8(img_buffer)
}

fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    // add point
    let entity_id = mini_gpu.world.add_entity(Entity::new());
    sprite_entity::make_mesh(
        glam::Vec3::new(0.0, 0.0, 0.0),
        &mini_gpu.renderer,
        &mut mini_gpu.world,
        entity_id,
    );
    let bytes = include_bytes!("../../case.jpg");
    let img = image::load_from_memory(bytes).unwrap();
    let texture = texture::Texture::from_image(
        &mini_gpu.renderer.device,
        &mini_gpu.renderer.queue,
        &img,
        Some("test"),
    )
    .unwrap();
    sprite_entity::make_material(
        &mini_gpu.renderer,
        &mut mini_gpu.world,
        SpriteMaterialConfig {
            width: 200.0 / mini_gpu.config.width as f32,
            height: 200.0 / mini_gpu.config.width as f32,
            radial: true,
            texture: Some(texture),
            ..Default::default()
        },
        entity_id,
    );
    let material_id = &mini_gpu
        .world
        .get_entity_component_index(entity_id, "material");

    let count = 1000;
    let range = 10.;
    for i in 0..count {
        let entity_id = mini_gpu.world.add_entity(Entity::new());
        let position = glam::Vec3::new(
            ((i as f32 / count as f32) - 0.5) * range,
            ((i as f32 / count as f32) - 0.5) * range,
            (random() as f32) * range,
        );

        sprite_entity::make_mesh(position, &mini_gpu.renderer, &mut mini_gpu.world, entity_id);

        mini_gpu
            .world
            .set_entity_component_index(entity_id, *material_id, "material");
    }
    //add line
    // let entity_line_id = mini_gpu.world.add_entity(Entity::new());
    // entity::mesh_line::make_mesh(
    //     &vec![
    //         glam::Vec3::new(0.2, 0.0, 0.0),
    //         glam::Vec3::new(0.8, 0.0, 0.0),
    //     ],
    //     30.0 as f32 / mini_gpu.config.width as f32,
    //     &mini_gpu.renderer,
    //     &mut mini_gpu.world,
    //     entity_line_id,
    // );
    // entity::mesh_line::make_material(
    //     &mini_gpu.renderer,
    //     &mut mini_gpu.world,
    //     vec![0.0, 1.0, 1.0, 1.0],
    //     entity_line_id,
    // );
}

async fn make_obj_mesh(mini_gpu: &mut MiniGPU) {
    // let path = std::path::Path::new("examples/models/cube/cube.obj");
    // let obj = utils::obj::load_obj_by_url(path,map, mini_gpu).await;
    // match obj {
    //     Ok(size) => {
    //         println!("Loaded obj with {} vertices", size);
    //     }
    //     Err(e) => {
    //         println!("Failed to load obj ({:?})", e,);
    //     }
    // }

    let camera = mini_gpu.world.get_default_camera().unwrap();
    let perspective_camera = camera.as_any().downcast_mut::<PerspectiveCamera>().unwrap();
    perspective_camera.config.position.z = 10.0;
    camera.update_bind_group(&mini_gpu.renderer);
}
