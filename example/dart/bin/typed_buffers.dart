import '../lib/bindings.dart';

void main() async {
  final api = Api.load();
  final buffers = <dynamic>[
    api.u8(20),
    api.u16(20),
    api.u32(20),
    api.u64(20),
    api.i8(20),
    api.i16(20),
    api.i32(20),
    api.i64(20),
    api.f32(20),
    api.f64(20),
  ];
  final views = buffers.map((b) => b.asTypedList());
  views.forEach((v) {
    assert(v.length == 20, v.toString());
    for (var i=0; i<20; i++) {
      assert(v[i] == i, v.toString());
    }
  });
  print('ok');
}

Iterable<int> get positiveIntegers sync* {
  int i = 0;
  while (true) yield i++;
}
