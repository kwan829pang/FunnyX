import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

class CorpOnboardController extends GetxController {
  final loading = false.obs;
  final error = RxnString();
  final corps = <Map<String, dynamic>>[].obs;
  final games = <Map<String, dynamic>>[].obs;

  AdminSystemProvider get _api => Get.find<AdminSystemProvider>();

  Future<void> load() async {
    loading.value = true;
    error.value = null;
    try {
      final results = await Future.wait([
        _api.listCorporateUsers(),
        _api.listAdminGames(),
      ]);
      corps.assignAll(results[0]);
      games.assignAll(results[1]);
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
    } finally {
      loading.value = false;
    }
  }

  Future<void> approveCorp(int id) async {
    await _api.approveCorporateUser(id);
    await load();
  }

  Future<void> rejectCorp(int id) async {
    await _api.rejectCorporateUser(id, reason: 'rejected by admin');
    await load();
  }

  Future<void> approveGame(int id) async {
    await _api.approveGame(id);
    await load();
  }

  Future<void> rejectGame(int id) async {
    await _api.rejectGame(id, reason: 'rejected by admin');
    await load();
  }
}
