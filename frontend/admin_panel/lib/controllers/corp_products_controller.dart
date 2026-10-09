import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

class CorpProductsController extends GetxController {
  final products = <AdminCorpProduct>[].obs;
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
      products.assignAll(
        await _api.listCorpProducts(status: statusFilter.value),
      );
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

  Future<void> setStatus(AdminCorpProduct product, String status) async {
    if (busy.value) return;
    busy.value = true;
    try {
      final updated = await _api.setCorpProductStatus(
        id: product.id,
        status: status,
      );
      final i = products.indexWhere((p) => p.id == product.id);
      if (i >= 0) {
        if (statusFilter.value != null && updated.status != statusFilter.value) {
          products.removeAt(i);
        } else {
          products[i] = updated;
        }
      }
      toastification.show(
        title: Text('${product.code} → $status'),
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
}
