pub struct Viewport {
    pub width: f32,
    pub height: f32,
    pub aspect: f32,
    pub scale_factor: f32,
}

impl Viewport {
    // todo: pass camera props to calculate pixel size
    pub fn new(width: u32, height: u32, scale_factor: f32) -> Viewport {
        let width_f32 = width as f32;
        let height_f32 = height as f32;
        Viewport {
            width: width_f32,
            height: height_f32,
            aspect: width_f32 / height_f32,
            scale_factor,
        }
    }
}
