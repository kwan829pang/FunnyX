import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

class PackagesController extends GetxController {
  final packages = <AdminShopPackage>[].obs;
  final loading = false.obs;
  final busy = false.obs;

  AdminSystemProvider get _api => Get.find<AdminSystemProvider>();

  @override
  void onInit() {
    super.onInit();
    reload();
  }

  Future<void> reload() async {
    loading.value = true;
    try {
      packages.assignAll(await _api.listPackages());
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

  Future<void> savePrice(AdminShopPackage pkg, String priceText) async {
    if (busy.value) return;
    final price = double.tryParse(priceText.trim());
    if (price == null || price <= 0) {
      toastification.show(
        title: const Text('Invalid fiat price'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 2),
      );
      return;
    }
    busy.value = true;
    try {
      final updated = await _api.updatePackage(id: pkg.id, fiatPrice: price);
      final i = packages.indexWhere((p) => p.id == pkg.id);
      if (i >= 0) packages[i] = updated;
      toastification.show(
        title: Text('Updated ${pkg.code}'),
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

  Future<void> setStatus(AdminShopPackage pkg, String status) async {
    if (busy.value) return;
    busy.value = true;
    try {
      final updated = await _api.setPackageStatus(id: pkg.id, status: status);
      final i = packages.indexWhere((p) => p.id == pkg.id);
      if (i >= 0) packages[i] = updated;
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

  Future<bool> createPackage({
    required String code,
    required String name,
    required String coinAmountText,
    required String fiatPriceText,
  }) async {
    if (busy.value) return false;
    final coin = double.tryParse(coinAmountText.trim());
    final price = double.tryParse(fiatPriceText.trim());
    if (coin == null || coin <= 0 || price == null || price <= 0) {
      toastification.show(
        title: const Text('Invalid coin amount or fiat price'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 2),
      );
      return false;
    }
    busy.value = true;
    try {
      await _api.createPackage(
        code: code.trim(),
        name: name.trim(),
        coinAmount: coin,
        fiatPrice: price,
        status: 'active',
      );
      await reload();
      toastification.show(
        title: const Text('Package created'),
        type: ToastificationType.success,
        autoCloseDuration: const Duration(seconds: 2),
      );
      return true;
    } catch (e) {
      toastification.show(
        title: Text('$e'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 3),
      );
      return false;
    } finally {
      busy.value = false;
    }
  }
}
