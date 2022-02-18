fn hello_world();

fn async_hello_world() -> Future<Result<u8>>;

fn get_image() -> buffer<u8>;

fn create(n: usize) -> DataTest;
object DataTest {
    fn get_copy() -> Vec<u8>;
    fn get_shmem() -> buffer<u8>;
}

fn get_u8_counting(n: usize) -> buffer<u8>;
fn get_u16_counting(n: usize) -> buffer<u16>;
fn get_u32_counting(n: usize) -> buffer<u32>;
fn get_u64_counting(n: usize) -> buffer<u64>;
fn get_i8_counting(n: usize) -> buffer<i8>;
fn get_i16_counting(n: usize) -> buffer<i16>;
fn get_i32_counting(n: usize) -> buffer<i32>;
fn get_i64_counting(n: usize) -> buffer<i64>;
fn get_f32_counting(n: usize) -> buffer<f32>;
fn get_f64_counting(n: usize) -> buffer<f64>;

fn create_list() -> Vec<CustomType>;

object CustomType {
    fn get_n() -> i32;
}

