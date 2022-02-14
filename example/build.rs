use ffi_gen::{Abi, FfiGen};
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let path = dir.join("api.rsh");
    println!(
        "cargo:rerun-if-changed={}",
        path.as_path().to_str().unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        dir.join("build.rs").as_path().to_str().unwrap()
    );
    let ffigen = FfiGen::new(&path).unwrap();
    // let rust = dir.join("src").join("bindings.rs");
    // std::fs::write(rust, format!("use crate::*;\nuse futures::Stream;\n{}", ffigen.generate_rust(Abi::Native64).unwrap())).unwrap();
    let dart = dir.join("dart").join("lib").join("bindings.dart");
    ffigen.generate_dart(dart, "api", "api").unwrap();
}
