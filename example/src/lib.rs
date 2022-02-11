use std::ops::Deref;
use futures::{Stream, StreamExt};

ffi_gen_macro::ffi_gen!("example/api.rsh");

fn log(msg: &str) {
    println!("{}", msg);
}

#[derive(Debug, Clone, Copy)]
pub struct OurStruct {
    pub x: u32,
    pub y: u32,
}

impl ToString for OurStruct {
    fn to_string(&self) -> String {
        format!("OurStruct({}, {})", self.x, self.y)
    }
}

impl OurStruct {
    fn print(&self) {
        log(&self.to_string());
    }

    fn get_x(&self) -> u32 {
        self.x
    }

    fn get_y(&self) -> u32 {
        self.y
    }
}

pub fn create_s(x: u32, y: u32) -> OurStruct {
    OurStruct { x, y }
}

pub struct OurStructList {
    pub list: Vec<OurStruct>,
}

pub fn new_struct_list() -> OurStructList {
    OurStructList { list: vec![] }
}

impl OurStructList {
    pub fn add(&mut self, s: Box<OurStruct>) {
        self.list.push(s.deref().clone());
    }

    pub fn print(&self) {
        for s in &self.list {
            s.print();
        }
    }

    pub fn get(&self, index: usize) -> Option<&OurStruct> {
        self.list.get(index)
    }
}

pub fn print_ss(ss: Box<OurStructList>) {
    ss.list.iter().for_each(|s| s.print());
}

pub fn create_ss() -> OurStructList {
    OurStructList { list: (0..10).map(move |i| OurStruct { x: i, y: i }).collect::<Vec<_>>() }
}
