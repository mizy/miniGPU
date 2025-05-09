pub trait LightTrait {
    fn get_buffer(&self) -> &wgpu::Buffer;

    fn get_bind_index(&self) -> u32;

    fn as_any(&self) -> &dyn std::any::Any;

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
