import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import 'cart_controller.dart';

class ShopCheckoutController extends GetxController {
  ShopCheckoutController(this.shop);

  final ShopDataProvider shop;

  final loading = false.obs;
  final submitting = false.obs;
  final error = RxnString();
  final bindings = <GameAccountBinding>[].obs;
  final selectedBindingId = RxnInt();
  final lastCreatedOrders = <ShopOrder>[].obs;

  Future<void> loadBindings() async {
    loading.value = true;
    error.value = null;
    try {
      final rows = await shop.listGameAccounts();
      bindings.assignAll(rows);
      if (rows.isNotEmpty && selectedBindingId.value == null) {
        selectedBindingId.value = rows.first.id;
      }
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
    } finally {
      loading.value = false;
    }
  }

  /// Creates one pending order per cart line qty unit (API = one SKU per order).
  Future<List<ShopOrder>> placeOrders({String? returnUrl}) async {
    final cart = Get.find<CartController>();
    final gameAccountId = selectedBindingId.value;
    if (gameAccountId == null) {
      throw const ApiError('Select a game account binding');
    }
    if (cart.lines.isEmpty) {
      throw const ApiError('Cart is empty');
    }
    submitting.value = true;
    error.value = null;
    final created = <ShopOrder>[];
    try {
      for (final line in cart.lines.toList()) {
        for (var i = 0; i < line.qty; i++) {
          final order = await shop.createOrder(
            sellerType: line.sellerType,
            packageId: line.sellerType == 'platform' ? line.catalogId : null,
            corpProductId: line.sellerType == 'corp' ? line.catalogId : null,
            gameAccountId: gameAccountId,
            returnUrl: returnUrl,
          );
          created.add(order);
        }
      }
      lastCreatedOrders.assignAll(created);
      cart.clear();
      return created;
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
      rethrow;
    } finally {
      submitting.value = false;
    }
  }
}
