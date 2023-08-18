use ::mini_gpu::{
    components::{
        material::{self, Material, MaterialConfig},
        mesh::Mesh,
    },
    entity::Entity,
    mini_gpu::MiniGPU,
    system::mesh_render::MeshRender,
    *,
};
use wasm_bindgen::prelude::*;
use winit::{event_loop::EventLoop, window::Window};

#[wasm_bindgen]
pub async fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
    log::info!("init wasm example");
    use winit::platform::web::WindowExtWebSys;
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    use winit::dpi::PhysicalSize;
    window.set_inner_size(PhysicalSize::new(1200, 800));

    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("wasm-example")?;
            let canvas = web_sys::Element::from(window.canvas());
            dst.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
    let size = window.inner_size();
    let mut mini_gpu = mini_gpu::MiniGPU::new(
        mini_gpu::MiniGPUConfig {
            width: size.width,
            height: size.height,
        },
        window,
    )
    .await;
    mini_gpu
        .scene
        .add_system("render".to_string(), Box::new(MeshRender {}));
    make_test_mesh(&mut mini_gpu);
    match mini_gpu.renderer.render(&mini_gpu.scene) {
        Ok(_) => log::info!("rendered"),
        Err(e) => log::error!("Failed to render: {}", e),
    }
}

fn make_test_mesh(mini_gpu: &mut MiniGPU) {
    let mesh = Mesh::new(
        vec![0.5, 0.5, -0.5, 0.5, 0., 0.],
        vec![0, 1, 2],
        &mini_gpu.renderer,
    );
    let material = Material::new(
        MaterialConfig {
            shader_text: include_str!("./mesh.wgsl").to_string(),
            topology: wgpu::PrimitiveTopology::TriangleList,
            uniforms: vec![1., 0., 0.5, 1.],
        },
        &mini_gpu.renderer,
    );
    //object1
    let entity_id = mini_gpu.scene.add_entity(Entity::new());
    mini_gpu
        .scene
        .set_entity_component::<Mesh>(entity_id, mesh, "mesh");
    mini_gpu
        .scene
        .set_entity_component::<Material>(entity_id, material, "material");
}
