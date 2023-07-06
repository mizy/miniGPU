pub mod material;
pub mod materials;
pub mod mesh;
pub struct ComponentRef<T> {
    Component: T,
}
impl<T> ComponentRef<T> {
    pub fn new(component: T) -> ComponentRef<T> {
        let instance = ComponentRef {
            Component: component,
        };
        instance
    }

    pub fn get_component(&self) -> &T {
        &self.Component
    }
}
