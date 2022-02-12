import 'dart:io';

import '../lib/bindings.dart';

void main() async {
  final api = Api.load();
  final img = await api.getImage();
  print(img.toUint8List().length);
  File.fromUri(Uri.parse('t.jpg')).writeAsBytes(img.toUint8List());
}
