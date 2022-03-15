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
