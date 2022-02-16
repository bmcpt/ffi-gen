import '../lib/bindings.dart';

void main() async {
  final api = Api.load();
  final u8s = api.getU8Counting(20).toUint8List();
  final u16s = api.getU16Counting(20).toUint16List();
  final u32s = api.getU32Counting(20).toUint32List();
  final u64s = api.getU64Counting(20).toUint64List();
  final i8s = api.getI8Counting(20).toInt8List();
  final i16s = api.getI16Counting(20).toInt16List();
  final i32s = api.getI32Counting(20).toInt32List();
  final i64s = api.getI64Counting(20).toInt64List();
  final f32s = api.getF32Counting(20).toFloat32List();
  final f64s = api.getF64Counting(20).toFloat64List();
  for (final i in positiveIntegers.take(20)) {
    assert(u8s[i] == i);
    assert(u16s[i] == i);
    assert(u32s[i] == i);
    assert(u64s[i] == i);
    assert(i8s[i] == i);
    assert(i16s[i] == i);
    assert(i32s[i] == i);
    assert(i64s[i] == i);
    assert(f32s[i] == i);
    assert(f64s[i] == i);
  }
  print('ok');
}

Iterable<int> get positiveIntegers sync* {
  int i = 0;
  while (true) yield i++;
}
