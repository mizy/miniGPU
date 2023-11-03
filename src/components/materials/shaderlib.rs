use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref SHADER_LIB: HashMap<String, String> = {
        let mut map = HashMap::new();
        map.insert(
            "CameraUniform".to_string(),
            include_str!("shaderlibs/camera_uniform.wgsl").to_string(),
        );

        map
    };
}
