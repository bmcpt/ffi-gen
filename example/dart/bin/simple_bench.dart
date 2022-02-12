import 'dart:io';

import '../lib/bindings.dart';

void main() async {
  final n = 1024 * 1024 * 10;

  final api = Api.load();
  final data = api.create(n);
  final t1 = DateTime.now();
  print(data.getCopy().length);
  final t2 = DateTime.now();
  print(t2.difference(t1).inMilliseconds);
  print(data.getShmem().toUint8List().length);
  final t3 = DateTime.now();
  print(t3.difference(t2).inMilliseconds);
}
