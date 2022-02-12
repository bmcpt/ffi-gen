
fn get_image() -> buffer;

fn create(n: usize) -> DataTest;
object DataTest {
    fn get_copy() -> Vec<u8>;
    fn get_shmem() -> buffer;
}