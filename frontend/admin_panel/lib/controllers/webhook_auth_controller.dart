import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

class WebhookAuthController extends GetxController {
  final items = <AdminWebhookPostAuth>[].obs;
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
      items.assignAll(await _api.listWebhookPostAuth());
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

  Future<bool> createItem({
    required String sourceCode,
    required String domainPattern,
    required String authKeySuffix,
    String? ppk,
    required String allowedIps,
  }) async {
    if (busy.value) return false;
    busy.value = true;
    try {
      final created = await _api.createWebhookPostAuth(
        sourceCode: sourceCode.trim(),
        domainPattern: domainPattern.trim(),
        authKeySuffix: authKeySuffix.trim(),
        ppk: ppk?.trim(),
        allowedIps: allowedIps.trim(),
      );
      items.insert(0, created);
      toastification.show(
        title: Text('Created ${created.sourceCode}'),
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

  Future<bool> updateItem({
    required int id,
    required String sourceCode,
    required String domainPattern,
    required String authKeySuffix,
    String? ppk,
    required String allowedIps,
  }) async {
    if (busy.value) return false;
    busy.value = true;
    try {
      final updated = await _api.updateWebhookPostAuth(
        id: id,
        sourceCode: sourceCode.trim(),
        domainPattern: domainPattern.trim(),
        authKeySuffix: authKeySuffix.trim(),
        ppk: ppk?.trim(),
        allowedIps: allowedIps.trim(),
      );
      final i = items.indexWhere((e) => e.id == id);
      if (i >= 0) items[i] = updated;
      toastification.show(
        title: Text('Updated ${updated.sourceCode}'),
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

  Future<void> setStatus(AdminWebhookPostAuth row, String status) async {
    if (busy.value) return;
    busy.value = true;
    try {
      final updated = await _api.setWebhookPostAuthStatus(
        id: row.id,
        status: status,
      );
      final i = items.indexWhere((e) => e.id == row.id);
      if (i >= 0) items[i] = updated;
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
