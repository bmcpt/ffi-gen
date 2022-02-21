import 'dart:io';

import '../lib/bindings.dart';

void main() async {
  final api = Api.load();
  final list = api.createList();
  for (final e in list) {
    print(e.getN());
  }

  list[0];
  print(api.sumList(list));
}
