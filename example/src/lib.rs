#![feature(vec_into_raw_parts)]
#![allow(unused)]

use anyhow::Result;
use std::io::Read;
use futures::Stream;

ffi_gen_macro::ffi_gen!("example/api.rsh");

const URL: &str = "https://file-examples-com.github.io/uploads/2017/10/file_example_JPG_1MB.jpg";

fn get_image() -> api::FfiBuffer<u8> {
    let mut bytes = vec![];
    ureq::get(URL).call().unwrap().into_reader().take(u64::MAX).read_to_end(&mut bytes).unwrap();
    api::FfiBuffer::new(bytes)
}

const RF: &str = "/dev/random";

struct DataTest {
    bytes: Vec<u8>,
}

fn create(n: usize) -> DataTest {
    let mut bytes = Vec::with_capacity(n);
    std::fs::File::open(RF).unwrap().take(n as u64).read_to_end(&mut bytes).unwrap();
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
        paste::paste! {
            fn [< get_ $ty _counting>](n: usize) -> api::FfiBuffer<$ty> {
                api::FfiBuffer::new((0..n).map(|n| n as $ty).collect())
            }
        }
    }}

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

// macro_rules! export_ffi_buffer_helpers {
//     ($ty:ident) => ({
//         #[no_mangle]
//         pub unsafe extern "C" fn __ffi_buffer_address(ptr: *mut c_void) -> *mut c_void {
//             let buffer = &*(ptr as *mut FfiBuffer<$ty>);
//             std::mem::transmute(buffer.bytes.as_ptr())
//         }
//
//         #[no_mangle]
//         pub unsafe extern "C" fn __ffi_buffer_size(ptr: *mut c_void) -> u32 {
//             let buffer = &*(ptr as *mut FfiBuffer<$ty>);
//             buffer.bytes.len() as u32
//         }
//     })
// }
//
// pub struct FfiBuffer<T> {
//     addr: *mut u8,
//     size: usize,
//     alloc: usize,
// }
//
// impl<T> FfiBuffer<T> {
//     fn new(data: Vec<T>) -> FfiBuffer<T> {
//         unsafe {
//             let (addr, size, alloc) = data.into_raw_parts();
//             FfiBuffer {
//                 addr: std::mem::transmute(addr),
//                 size: size * std::mem::size_of::<T>(),
//                 alloc: alloc * std::mem::size_of::<T>(),
//             }
//         }
//     }
// }
//
// impl<T> Drop for FfiBuffer<T> {
//     fn drop(&mut self) {
//         unsafe {
//             Vec::from_raw_parts(self.addr, self.size, self.alloc);
//         }
//     }
// }
//
// #[no_mangle]
// pub unsafe extern "C" fn __ffi_buffer_address(ptr: *mut c_void) -> *mut c_void {
//     let buffer = &*(ptr as *mut FfiBuffer<u8>);
//     buffer.addr
// }
//
// #[no_mangle]
// pub unsafe extern "C" fn __ffi_buffer_size(ptr: *mut c_void) -> u32 {
//     let buffer = &*(ptr as *mut FfiBuffer<u8>);
//     buffer.size as _
// }
