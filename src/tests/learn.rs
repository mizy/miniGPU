use std::collections::HashMap;

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

fn get_raw_point(map_ref: &mut HashMap<&str, usize>) {
    let a = A { id: 1 };
    let b = Box::new(a);
    map_ref.insert("a", Box::into_raw(b) as usize);
}

use std::any::Any;

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
