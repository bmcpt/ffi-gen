import '../lib/bindings.dart';

void main() {
  final api = Api.load();
  final list = api.newStructList();
  list.add(api.createS(1, 1));
  list.add(api.createS(2, 2));
  list.add(api.createS(3, 3));
  list.add(api.createS(4, 4));
  list.print();

  final list2 = api.createSs();
  api.printSs(list2);
}
