fn hello_world();

fn async_hello_world() -> Future<Result<u8>>;

fn get_image() -> buffer<u8>;

fn create(n: usize) -> DataTest;
object DataTest {
    fn get_copy() -> Vec<u8>;
    fn get_shmem() -> buffer<u8>;
}

fn u8(n: usize) -> buffer<u8>;
fn u16(n: usize) -> buffer<u16>;
fn u32(n: usize) -> buffer<u32>;
fn u64(n: usize) -> buffer<u64>;
fn i8(n: usize) -> buffer<i8>;
fn i16(n: usize) -> buffer<i16>;
fn i32(n: usize) -> buffer<i32>;
fn i64(n: usize) -> buffer<i64>;
fn f32(n: usize) -> buffer<f32>;
fn f64(n: usize) -> buffer<f64>;

fn create_list() -> Vec<CustomType>;

fn sum_list(l: Vec<CustomType>) -> u32;

object CustomType {
    fn get_n() -> i32;
}

fn create_custom_type(n: i32) -> CustomType;

fn s() -> string;

fn ss() -> Vec<string>;

object Vector2 {
    fn x() -> u64;
    fn y() -> u64;
}

object Vector3 {
    fn x() -> u64;
    fn y() -> u64;
    fn z() -> u64;
}

enum Shape {
    Square(Vector2),
    Cube(Vector3),
    None
}

fn get_shape() -> Shape;

fn get_shapes() -> Vec<Shape>;
