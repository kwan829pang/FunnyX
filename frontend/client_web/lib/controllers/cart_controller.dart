import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

class CartController extends GetxController {
  final lines = <CartLine>[].obs;

  int get itemCount => lines.fold(0, (sum, l) => sum + l.qty);

  double get subtotal => lines.fold(0.0, (sum, l) => sum + l.lineTotal);

  void addPackage(ShopPackage p) {
    _add(
      CartLine(
        sellerType: 'platform',
        catalogId: p.id,
        code: p.code,
        name: p.name,
        fiatPrice: p.fiatPrice,
      ),
    );
  }

  void addCorpProduct(CorpProduct p) {
    _add(
      CartLine(
        sellerType: 'corp',
        catalogId: p.id,
        code: p.code,
        name: p.name,
        fiatPrice: p.fiatPrice,
      ),
    );
  }

  void _add(CartLine line) {
    final i = lines.indexWhere((e) => e.key == line.key);
    if (i >= 0) {
      lines[i].qty += 1;
      lines.refresh();
    } else {
      lines.add(line);
    }
  }

  void setQty(String key, int qty) {
    if (qty <= 0) {
      remove(key);
      return;
    }
    final i = lines.indexWhere((e) => e.key == key);
    if (i >= 0) {
      lines[i].qty = qty;
      lines.refresh();
    }
  }

  void remove(String key) {
    lines.removeWhere((e) => e.key == key);
  }

  void clear() => lines.clear();
}
