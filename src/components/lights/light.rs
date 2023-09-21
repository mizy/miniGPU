pub struct LightRef {
    pub light: Box<dyn Light>,
}

pub trait Light {
    fn get_buffer(&self) -> &wgpu::Buffer;

    fn get_bind_index(&self) -> u32;
}

impl LightRef {
    pub fn new(light: Box<dyn Light>) -> LightRef {
        LightRef { light }
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        self.light.get_buffer()
    }

    pub fn get_bind_index(&self) -> u32 {
        self.light.get_bind_index()
    }

    pub fn get_light<T>(&self) -> &Box<T> {
        let t = &self.light as *const Box<dyn Light> as *mut Box<T>;
        unsafe { &*t }
    }

    pub fn get_light_mut<T>(&mut self) -> &mut Box<T> {
        let t = &mut self.light as *mut Box<dyn Light> as *mut Box<T>;
        unsafe { &mut *t }
    }
}

#[macro_export]
macro_rules! light_ref {
    ( $( $x:expr )? ) => {{
        LightRef::new(Box::new($($x)?))
    }};
}
