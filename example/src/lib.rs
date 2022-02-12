mod bindings;

use bindings::*;
use std::io::Read;

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