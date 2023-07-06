struct A {
    pub value: Vec<u32>,
    pub name: String,
}

#[test]
fn test_layout_ptr() {
    let layout = std::alloc::Layout::new::<A>();
    let ptr = unsafe { std::alloc::alloc(layout) } as *mut A;
    let a = {
        A {
            value: vec![1, 2, 3],
            name: "hello".to_string(),
        }
    };
    unsafe { std::ptr::write(ptr, a) };
    let read_a = unsafe { std::ptr::read(ptr as *const A) };
    print!("read_a: {:?}", read_a.name);
}
