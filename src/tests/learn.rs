use crate::scene::Scene;

pub struct A {
    pub id: u32,
}

pub trait B {
    fn get_id(&self) -> u32;
}
impl B for A {
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[test]
fn test_struct() {
    let a = Box::new(A { id: 1 });
    println!("=======>a:{:?}", &a as *const Box<A>);
    let layout = std::alloc::Layout::new::<Box<A>>();
    let ptr = unsafe {
        let ptr = std::alloc::alloc(layout) as *mut Box<A>;
        if ptr.is_null() {
            panic!("memory allocation failed");
        }
        std::ptr::write(ptr, a);
        ptr as *mut u8
    };
    let t = ptr as *mut Box<A>;
    let b = unsafe { &mut *t };
    println!("=======>b:{:?}", b.get_id());

    // println!("=======>new:{:?}", b.get_id());
}

trait Trait {
    fn display(&self);
}

struct Struct1 {
    value: u32,
}

impl Trait for Struct1 {
    fn display(&self) {
        println!("Struct1 with value: {}", self.value);
    }
}
struct Struct2 {
    value: u32,
}

impl Trait for Struct2 {
    fn display(&self) {
        println!("Struct2 with value: {}", self.value);
    }
}

#[test]
fn test_mat4_scalar() {
    let right = glam::Vec3::new(0.0, -1., -1.0);
    let up = glam::Vec3::new(0.0, 1.0, 0.);
    let forward = right.cross(up);
    println!("forward: {:?}", forward);
}

#[test]
fn test_dyn_box_trait() {
    let s1 = Box::new(Struct1 { value: 1 });
    s1.display();
    let mut scene = Scene::new();
    let index = scene.add_component::<Box<dyn Trait>>(s1);
    let s1 = scene.get_component::<Box<Struct1>>(index);
    s1.display();
}
