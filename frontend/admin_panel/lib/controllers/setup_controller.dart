import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import 'setup_middleware.dart';

class SetupController extends GetxController {
  final step = 0.obs;
  final busy = false.obs;
  final loading = true.obs;
  final baseFiat = 'HKD'.obs;
  final packagePrices = <int, TextEditingController>{}.obs;
  final setup = Rxn<AdminSetupStatus>();

  AdminSystemProvider get _api => Get.find<AdminSystemProvider>();

  @override
  void onInit() {
    super.onInit();
    refreshSetup();
  }

  @override
  void onClose() {
    for (final c in packagePrices.values) {
      c.dispose();
    }
    super.onClose();
  }

  Future<void> refreshSetup() async {
    loading.value = true;
    try {
      final s = await _api.fetchSetup();
      setup.value = s;
      if (s.baseFiatCurrency != null && s.baseFiatCurrency!.isNotEmpty) {
        baseFiat.value = s.baseFiatCurrency!;
      }
      for (final c in packagePrices.values) {
        c.dispose();
      }
      final map = <int, TextEditingController>{};
      for (final p in s.packages) {
        map[p.id] = TextEditingController(text: p.fiatPrice.toString());
      }
      packagePrices.assignAll(map);
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

  Future<void> saveBaseCurrency() async {
    if (busy.value) return;
    busy.value = true;
    try {
      await _api.setBaseCurrency(baseFiat.value);
      await refreshSetup();
      step.value = 2;
      toastification.show(
        title: const Text('Base currency saved'),
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

  Future<void> savePackages() async {
    if (busy.value) return;
    busy.value = true;
    try {
      final packages = setup.value?.packages ?? const <AdminShopPackage>[];
      for (final p in packages) {
        final text = packagePrices[p.id]?.text.trim() ?? '';
        final price = double.tryParse(text);
        if (price == null || price <= 0) {
          throw ApiError('Invalid price for ${p.code}');
        }
        await _api.updatePackage(id: p.id, fiatPrice: price, status: 'active');
      }
      await refreshSetup();
      step.value = 3;
      toastification.show(
        title: const Text('Package prices saved'),
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

  Future<void> complete() async {
    if (busy.value) return;
    busy.value = true;
    try {
      final s = await _api.completeSetup();
      setup.value = s;
      SetupMiddleware.markInitialized(true);
      toastification.show(
        title: const Text('System initialized'),
        type: ToastificationType.success,
        autoCloseDuration: const Duration(seconds: 2),
      );
      Get.offAllNamed('/dashboard');
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
