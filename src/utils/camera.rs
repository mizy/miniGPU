use crate::{
    components::{
        orthographic_camera::{OrthographicCamera, OrthographicCameraConfig},
        perspective_camera::CameraTrait,
    },
    entity,
    mini_gpu::MiniGPU,
};

// default z is 10.0
pub fn default_orthographic_camera(mini_gpu: &mut MiniGPU) {
    let mut camera = OrthographicCamera::new(
        OrthographicCameraConfig {
            ..Default::default()
        },
        &mini_gpu.renderer,
    );
    camera.config.position.z = 10.0;
    let entity_id = mini_gpu.scene.add_entity(entity::Entity::new());
    mini_gpu.scene.set_entity_component::<Box<dyn CameraTrait>>(
        entity_id,
        Box::new(camera),
        "camera",
    );
    mini_gpu.scene.default_camera = Some(entity_id);
}
