use anyhow::Result;
use std::io::Read;
use futures::Stream;

ffi_gen_macro::ffi_gen!("example/api.rsh");

const URL: &str = "https://file-examples-com.github.io/uploads/2017/10/file_example_JPG_1MB.jpg";

fn get_image() -> api::FfiBuffer {
    let mut bytes = vec![];
    ureq::get(URL).call().unwrap().into_reader().take(u64::MAX).read_to_end(&mut bytes).unwrap();
    api::FfiBuffer { bytes }
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

    fn get_shmem(&self) -> api::FfiBuffer {
        api::FfiBuffer {
            bytes: self.bytes.clone()
        }
    }
}

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