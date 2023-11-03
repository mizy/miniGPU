pub trait LightTrait {
    fn get_buffer(&self) -> &wgpu::Buffer;

    fn get_bind_index(&self) -> u32;
}
