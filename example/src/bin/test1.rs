fn main() {
    unsafe {
        f();
    }
}

#[derive(Debug)]
struct CustomType {
    n: i32,
}

impl CustomType {
    fn get_n(&self) -> i32 {
        self.n
    }
}

unsafe fn f() {
    let mut v = Vec::<CustomType>::new();
    v.push(CustomType { n: 4 });
    v.push(CustomType { n: 2 });

    let ptr = v.get(0).unwrap() as *const _;
    let b = Box::from_raw(std::mem::transmute::<_, *mut CustomType>(ptr));
    dbg!(&b);
    Box::into_raw(b);
}