use std::{collections::HashMap, hash::Hash, ptr, thread::sleep};

pub struct A {
    pub id: u32,
}

#[test]
fn test_struct() {
    let addr: usize;
    {
        let mut map: HashMap<&str, usize> = HashMap::new();
        get_raw_point(&mut map);
        addr = *map.get("a").unwrap();
        map.remove("a");
    }
    {
        let mut b = unsafe { Box::from_raw(addr as *mut A) };
        b.id = 2;
    }

    let mut b = unsafe { Box::from_raw(addr as *mut A) };
    b.id = 2;
    println!("=======>new:{:?}", b.id);
}

fn get_raw_point(map_ref: &mut HashMap<&str, usize>) {
    let a = A { id: 1 };
    let b = Box::new(a);
    map_ref.insert("a", Box::into_raw(b) as usize);
}
