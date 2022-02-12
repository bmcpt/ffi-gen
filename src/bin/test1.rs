use std::path::Path;
use ffi_gen::FfiGen;

fn main() {
    let ffi = FfiGen::new(Path::new(file!()).parent().unwrap().join("api.rsh").to_str().unwrap()).unwrap();
}