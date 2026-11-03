import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import '../services/auth_failure_handler.dart';

class TradeController extends GetxController {
  final loading = false.obs;
  final busy = false.obs;
  final markets = <ExchangeMarket>[].obs;
  final orders = <ExchangeOrder>[].obs;
  final selected = Rxn<ExchangeMarket>();
  final bookText = ''.obs;
  final gameAccounts = <GameAccountBinding>[].obs;

  ExchangeDataProvider get _api => Get.find<ExchangeDataProvider>();
  ShopDataProvider get _shop => Get.find<ShopDataProvider>();
  SessionController get _session => Get.find<SessionController>();

  @override
  void onInit() {
    super.onInit();
    if (_session.isAuthenticated) {
      reload();
    }
  }

  Future<void> reload() async {
    if (!_session.isAuthenticated) {
      return;
    }
    loading.value = true;
    try {
      final results = await Future.wait([
        _api.listMarkets(),
        _api.listOrders(),
        _shop.listGameAccounts(),
      ]);
      markets.assignAll(results[0] as List<ExchangeMarket>);
      orders.assignAll(results[1] as List<ExchangeOrder>);
      gameAccounts.assignAll(results[2] as List<GameAccountBinding>);
      if (selected.value == null && markets.isNotEmpty) {
        selected.value = markets.first;
        await loadBook();
      }
    } catch (e) {
      if (!AuthFailureHandler.isAuthFailure(e)) {
        toastification.show(
          title: Text('$e'),
          type: ToastificationType.error,
          autoCloseDuration: const Duration(seconds: 3),
        );
      }
    } finally {
      loading.value = false;
    }
  }

  Future<void> selectMarket(ExchangeMarket m) async {
    selected.value = m;
    await loadBook();
  }

  Future<void> loadBook() async {
    final m = selected.value;
    if (m == null) return;
    try {
      final data = await _api.getBook(m.id);
      bookText.value = data['book']?.toString() ?? data.toString();
    } catch (e) {
      if (AuthFailureHandler.isAuthFailure(e)) {
        bookText.value = '';
        return;
      }
      bookText.value = '$e';
    }
  }

  Future<void> place({
    required String side,
    required String orderType,
    required String qtyText,
    required String priceText,
    required int gameAccountId,
  }) async {
    final m = selected.value;
    if (m == null || busy.value) return;
    final qty = int.tryParse(qtyText.trim());
    if (qty == null || qty <= 0) {
      toastification.show(
        title: const Text('Invalid quantity'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 2),
      );
      return;
    }
    var price = 0;
    if (orderType == 'price') {
      price = int.tryParse(priceText.trim()) ?? 0;
      if (price <= 0) {
        toastification.show(
          title: const Text('Invalid price'),
          type: ToastificationType.error,
          autoCloseDuration: const Duration(seconds: 2),
        );
        return;
      }
    }
    busy.value = true;
    try {
      final order = await _api.placeOrder(
        symbol: m.symbol,
        side: side,
        orderType: orderType,
        quantity: qty,
        price: price,
        gameAccountId: gameAccountId,
        marketPairId: m.id,
      );
      toastification.show(
        title: Text('Order #${order.id} ${order.status}'),
        type: ToastificationType.success,
        autoCloseDuration: const Duration(seconds: 2),
      );
      await reload();
    } catch (e) {
      if (!AuthFailureHandler.isAuthFailure(e)) {
        toastification.show(
          title: Text('$e'),
          type: ToastificationType.error,
          autoCloseDuration: const Duration(seconds: 3),
        );
      }
    } finally {
      busy.value = false;
    }
  }

  Future<void> cancel(ExchangeOrder o) async {
    if (busy.value) return;
    busy.value = true;
    try {
      await _api.cancelOrder(o.id);
      await reload();
    } catch (e) {
      if (!AuthFailureHandler.isAuthFailure(e)) {
        toastification.show(
          title: Text('$e'),
          type: ToastificationType.error,
          autoCloseDuration: const Duration(seconds: 3),
        );
      }
    } finally {
      busy.value = false;
    }
  }
}
