use ffi_gen::FfiGen;
use std::path::Path;

fn main() {
    let _ffi = FfiGen::new(
        Path::new(file!())
            .parent()
            .unwrap()
            .join("api.rsh")
            .to_str()
            .unwrap(),
    )
    .unwrap();
}
