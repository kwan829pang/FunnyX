import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import 'corp_session_controller.dart';

class CorpController extends GetxController {
  CorpController(this._api, this._session);

  final CorpDataProvider _api;
  final CorpSessionController _session;

  final loading = false.obs;
  final error = RxnString();
  final games = <CorpGame>[].obs;

  CorpProfile? get profile => _session.profile.value;

  Future<bool> register({
    required String companyName,
    required String contactName,
    required String contactEmail,
    required String password,
    required List<int> brBytes,
    required String brFileName,
  }) async {
    loading.value = true;
    error.value = null;
    try {
      final result = await _api.register(
        companyName: companyName,
        contactName: contactName,
        contactEmail: contactEmail,
        password: password,
        brBytes: brBytes,
        brFileName: brFileName,
      );
      _session.setAuth(result);
      return true;
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
      return false;
    } finally {
      loading.value = false;
    }
  }

  Future<bool> login({required String login, required String password}) async {
    loading.value = true;
    error.value = null;
    try {
      final result = await _api.login(login: login, password: password);
      _session.setAuth(result);
      return true;
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
      return false;
    } finally {
      loading.value = false;
    }
  }

  Future<void> refreshProfile() async {
    loading.value = true;
    error.value = null;
    try {
      _session.profile.value = await _api.profile();
      games.assignAll(await _api.listGames());
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
    } finally {
      loading.value = false;
    }
  }

  Future<CorpGame?> submitGame({
    required String gameName,
    required String gameCode,
    String? partnerCode,
    String? partnerGameId,
    String? description,
    List<int>? governmentBytes,
    String? governmentFileName,
    List<int>? materialBytes,
    String? materialFileName,
  }) async {
    loading.value = true;
    error.value = null;
    try {
      final game = await _api.submitGame(
        gameName: gameName,
        gameCode: gameCode,
        partnerCode: partnerCode,
        partnerGameId: partnerGameId,
        description: description,
        governmentBytes: governmentBytes,
        governmentFileName: governmentFileName,
        materialBytes: materialBytes,
        materialFileName: materialFileName,
      );
      await refreshProfile();
      return game;
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
      return null;
    } finally {
      loading.value = false;
    }
  }

  void logout() {
    _session.clear();
    games.clear();
  }
}
