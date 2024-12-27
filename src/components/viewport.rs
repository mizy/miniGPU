use glam::Vec2;

pub struct Viewport {
    pub width: f32,
    pub height: f32,
    pub aspect: f32,
    // per pixel representation of the screen
    pub pixel_size: Vec2,
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Viewport {
        Viewport {
            width,
            height,
            aspect: width / height,
            pixel_size: Vec2::new(1.0 / width, 1.0 / height),
        }
    }
}
