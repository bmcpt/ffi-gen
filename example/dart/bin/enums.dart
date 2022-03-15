
import '../lib/bindings.dart';

void main() async {
  final api = Api.load();
  printShape(api.getShape());
  final shapes = api.getShapes();
  print(shapes.length);
  shapes.forEach(printShape);
}

void printShape(Shape shape) {
  switch (shape.tag) {
    case ShapeTag.Square:
      final size = shape.inner as Vector2;
      print("Square(${size.x()}, ${size.y()})");
      break;
    case ShapeTag.Cube:
      final size = shape.inner as Vector3;
      print("Cube(${size.x()}, ${size.y()}, ${size.z()})");
      break;
    case ShapeTag.None:
      print("None");
      break;
  }
}