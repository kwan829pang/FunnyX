import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

class MarketsController extends GetxController {
  final markets = <AdminMarket>[].obs;
  final loading = false.obs;
  final busy = false.obs;
  final statusFilter = RxnString();

  AdminSystemProvider get _api => Get.find<AdminSystemProvider>();

  @override
  void onInit() {
    super.onInit();
    reload();
  }

  Future<void> reload() async {
    loading.value = true;
    try {
      markets.assignAll(await _api.listMarkets(status: statusFilter.value));
    } catch (e) {
      toastification.show(
        title: Text('$e'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 3),
      );
    } finally {
      loading.value = false;
    }
  }

  Future<void> setFilter(String? status) async {
    statusFilter.value = status;
    await reload();
  }

  Future<void> approve(AdminMarket market) async {
    if (busy.value) return;
    busy.value = true;
    try {
      final result = await _api.approveMarket(market.id);
      _upsert(result.market);
      final engineNote = result.engineActivated
          ? 'engine activated'
          : 'DB approved (engine activate failed or offline)';
      toastification.show(
        title: Text('${result.market.marketName} approved — $engineNote'),
        type: result.engineActivated
            ? ToastificationType.success
            : ToastificationType.warning,
        autoCloseDuration: const Duration(seconds: 3),
      );
    } catch (e) {
      toastification.show(
        title: Text('$e'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 3),
      );
    } finally {
      busy.value = false;
    }
  }

  Future<void> reject(AdminMarket market) async {
    if (busy.value) return;
    busy.value = true;
    try {
      final updated = await _api.rejectMarket(market.id);
      _upsert(updated);
      toastification.show(
        title: Text('${updated.marketName} rejected'),
        type: ToastificationType.success,
        autoCloseDuration: const Duration(seconds: 2),
      );
    } catch (e) {
      toastification.show(
        title: Text('$e'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 3),
      );
    } finally {
      busy.value = false;
    }
  }

  void _upsert(AdminMarket updated) {
    final i = markets.indexWhere((m) => m.id == updated.id);
    if (i < 0) {
      markets.insert(0, updated);
      return;
    }
    if (statusFilter.value != null && updated.status != statusFilter.value) {
      markets.removeAt(i);
    } else {
      markets[i] = updated;
    }
  }
}
