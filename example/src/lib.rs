#![feature(vec_into_raw_parts)]
#![allow(unused, clippy::transmutes_expressible_as_ptr_casts)]

use anyhow::Result;
use futures::Stream;
use std::io::Read;

ffi_gen_macro::ffi_gen!("example/api.rsh");

const URL: &str = "https://file-examples-com.github.io/uploads/2017/10/file_example_JPG_1MB.jpg";

fn get_image() -> api::FfiBuffer<u8> {
    let mut bytes = vec![];
    ureq::get(URL)
        .call()
        .unwrap()
        .into_reader()
        .take(u64::MAX)
        .read_to_end(&mut bytes)
        .unwrap();
    api::FfiBuffer::new(bytes)
}

struct DataTest {
    bytes: Vec<u8>,
}

fn create(n: usize) -> DataTest {
    let mut bytes = Vec::with_capacity(n);
    for _ in 0..n {
        bytes.push(42);
    }
    DataTest { bytes }
}

impl DataTest {
    fn get_copy(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    fn get_shmem(&self) -> api::FfiBuffer<u8> {
        api::FfiBuffer::new(self.bytes.clone())
    }
}

macro_rules! gen_counting_func {
            ($ty:ident) => {
                pub fn $ty(n: usize) -> api::FfiBuffer<$ty> {
                    api::FfiBuffer::new((0..n).map(|n| n as $ty).collect())
                }
            };
        }

gen_counting_func!(u8);
gen_counting_func!(u16);
gen_counting_func!(u32);
gen_counting_func!(u64);
gen_counting_func!(i8);
gen_counting_func!(i16);
gen_counting_func!(i32);
gen_counting_func!(i64);
gen_counting_func!(f32);
gen_counting_func!(f64);

#[cfg(target_family = "wasm")]
extern "C" {
    fn __console_log(ptr: isize, len: usize);
}

fn log(msg: &str) {
    #[cfg(target_family = "wasm")]
    return unsafe { __console_log(msg.as_ptr() as _, msg.len()) };
    #[cfg(not(target_family = "wasm"))]
    println!("{}", msg);
}

pub fn hello_world() {
    log("hello world");
}

pub async fn async_hello_world() -> Result<u8> {
    log("hello world");
    Ok(0)
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

fn create_list() -> Vec<CustomType> {
    vec![
        CustomType { n: 5 },
        CustomType { n: 4 },
        CustomType { n: 3 },
        CustomType { n: 2 },
        CustomType { n: 1 },
    ]
}

fn sum_list(l: &[CustomType]) -> u32 {
    l.iter().map(|e| e.n).sum::<i32>() as _
}

fn s() -> String {
    "string from rust".to_string()
}

fn ss() -> Vec<String> {
    vec![
        "first",
        "second",
        "third",
    ].iter().map(|s| s.to_string()).collect()
}