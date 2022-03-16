import 'dart:io';

import '../lib/bindings.dart';

void main() async {
  final api = Api.load();
  final list = api.createList();
  final el = list.remove(2);
  list.insert(0, el);
  list.add(api.createCustomType(10));
  list.add(api.createCustomType(20));
  for (final e in list) {
    print(e.getN());
  }

  print(api.sumList(list));
}
