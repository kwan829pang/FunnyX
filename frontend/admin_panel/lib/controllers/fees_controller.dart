import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

class FeesController extends GetxController {
  final fees = <AdminPlatformFee>[].obs;
  final corps = <AdminFeeCorp>[].obs;
  final catalog = <String, double>{}.obs;
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
      final result = await _api.listPlatformFees();
      fees.assignAll(result.fees);
      catalog.assignAll(result.catalog);
      corps.assignAll(await _api.listFeeCorps());
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

  Future<bool> createFee({
    required int corporateUserId,
    required String feeType,
    required double amount,
    required String payCurrency,
    int? licenseYear,
  }) async {
    if (busy.value) return false;
    busy.value = true;
    try {
      final created = await _api.createPlatformFee(
        corporateUserId: corporateUserId,
        feeType: feeType,
        amount: amount,
        payCurrency: payCurrency,
        licenseYear: licenseYear,
      );
      fees.insert(0, created);
      toastification.show(
        title: Text('Fee #${created.id} created'),
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

  Future<bool> setApiEnabled(AdminFeeCorp corp, bool enabled) async {
    if (busy.value) return false;
    busy.value = true;
    try {
      final updated = await _api.patchCorpAccess(
        id: corp.id,
        apiEnabled: enabled,
      );
      final i = corps.indexWhere((c) => c.id == corp.id);
      if (i >= 0) corps[i] = updated;
      toastification.show(
        title: Text(
          enabled
              ? 'APIs enabled for ${updated.companyName}'
              : 'APIs disabled for ${updated.companyName}',
        ),
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

  Future<bool> setLicenseDeadline(AdminFeeCorp corp, int deadlineMs) async {
    if (busy.value) return false;
    busy.value = true;
    try {
      final updated = await _api.patchCorpAccess(
        id: corp.id,
        licenseDeadlineAt: deadlineMs,
      );
      final i = corps.indexWhere((c) => c.id == corp.id);
      if (i >= 0) corps[i] = updated;
      toastification.show(
        title: Text('License deadline updated'),
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
