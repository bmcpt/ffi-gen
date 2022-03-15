#![feature(vec_into_raw_parts)]

use ffi_gen::{Abi, FfiGen};
use std::path::Path;

fn main() {
    let dir = Path::new(file!()).parent().unwrap();
    let ffi = FfiGen::new(dir.join("api.rsh").to_str().unwrap()).unwrap();
    let rs = ffi.generate_rust(Abi::Native64).unwrap();
    std::fs::write(dir.join("bindings.rs"), rs).unwrap();
}
