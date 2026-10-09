import 'dart:async';

import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

class ShopOrdersController extends GetxController {
  ShopOrdersController(this.shop);

  final ShopDataProvider shop;

  final loading = false.obs;
  final error = RxnString();
  final orders = <ShopOrder>[].obs;
  final detail = Rxn<ShopOrder>();
  Timer? _poll;

  @override
  void onClose() {
    _poll?.cancel();
    super.onClose();
  }

  Future<void> loadOrders({bool pendingOnly = false}) async {
    loading.value = true;
    error.value = null;
    try {
      var list = await shop.listOrders();
      if (pendingOnly) {
        list = list.where((o) => o.isPending).toList();
      }
      orders.assignAll(list);
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
    } finally {
      loading.value = false;
    }
  }

  Future<void> loadDetail(int id) async {
    loading.value = true;
    error.value = null;
    try {
      detail.value = await shop.getOrder(id);
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
    } finally {
      loading.value = false;
    }
  }

  void startDetailPoll(int id) {
    _poll?.cancel();
    _poll = Timer.periodic(const Duration(seconds: 5), (_) async {
      try {
        final o = await shop.getOrder(id);
        detail.value = o;
        if (!o.isPending) {
          _poll?.cancel();
        }
      } catch (_) {}
    });
  }

  void startPendingPoll() {
    _poll?.cancel();
    _poll = Timer.periodic(const Duration(seconds: 8), (_) async {
      try {
        final list = await shop.listOrders();
        orders.assignAll(list.where((o) => o.isPending).toList());
      } catch (_) {}
    });
  }

  void stopPoll() {
    _poll?.cancel();
    _poll = null;
  }

  Future<void> cancel(int id) async {
    error.value = null;
    try {
      final o = await shop.cancelOrder(id);
      final i = orders.indexWhere((e) => e.id == id);
      if (i >= 0) {
        orders[i] = o;
        orders.refresh();
      }
      if (detail.value?.id == id) {
        detail.value = o;
      }
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
      rethrow;
    }
  }
}
