import 'dart:io';

import '../lib/bindings.dart';

void main() async {
  var l = [];
  final api = Api.load();
  final list = api.createList();
  for (int i=0; i<list.length; i++) {
    print(list.elementAt(i).getN());
  }
}
