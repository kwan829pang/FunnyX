import 'dart:async';

import 'package:get/get.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../api/token_store.dart';

/// GetX session state: access token from Session Token Server.
///
/// Persists access (and optional refresh) so Flutter Web refresh / hot restart
/// does not leave the UI on protected routes with a null Bearer.
class SessionController extends GetxController implements TokenStore {
  SessionController({
    this.prefsAccessKey = 'fx_session_access',
    this.prefsRefreshKey = 'fx_session_refresh',
  });

  final String prefsAccessKey;
  final String prefsRefreshKey;

  final RxnString _token = RxnString();
  final RxnString _refreshToken = RxnString();
  SharedPreferences? _prefs;
  bool _restored = false;

  @override
  String? get token => _token.value;

  String? get refreshToken => _refreshToken.value;

  RxnString get tokenRx => _token;

  bool get restored => _restored;

  @override
  bool get isAuthenticated {
    final value = _token.value;
    return value != null && value.isNotEmpty;
  }

  /// Load tokens from disk before routing (call from `main` after bindings).
  Future<void> restore() async {
    _prefs = await SharedPreferences.getInstance();
    final access = _prefs!.getString(prefsAccessKey);
    final refresh = _prefs!.getString(prefsRefreshKey);
    if (access != null && access.isNotEmpty) {
      _token.value = access;
      _refreshToken.value =
          (refresh != null && refresh.isNotEmpty) ? refresh : null;
    }
    _restored = true;
  }

  /// Set access (+ optional refresh) and persist.
  void setSession({required String accessToken, String? refreshToken}) {
    final access = accessToken.trim();
    _token.value = access.isEmpty ? null : access;
    final refresh = refreshToken?.trim();
    _refreshToken.value =
        (refresh != null && refresh.isNotEmpty) ? refresh : null;
    unawaited(_persist());
  }

  @override
  void setToken(String? value) {
    final access = value?.trim();
    _token.value = (access != null && access.isNotEmpty) ? access : null;
    if (_token.value == null) {
      _refreshToken.value = null;
    }
    unawaited(_persist());
  }

  void clear() {
    _token.value = null;
    _refreshToken.value = null;
    unawaited(_persist());
  }

  Future<void> _persist() async {
    try {
      final prefs = _prefs ?? await SharedPreferences.getInstance();
      _prefs = prefs;
      final access = _token.value;
      if (access == null || access.isEmpty) {
        await prefs.remove(prefsAccessKey);
        await prefs.remove(prefsRefreshKey);
        return;
      }
      await prefs.setString(prefsAccessKey, access);
      final refresh = _refreshToken.value;
      if (refresh == null || refresh.isEmpty) {
        await prefs.remove(prefsRefreshKey);
      } else {
        await prefs.setString(prefsRefreshKey, refresh);
      }
    } catch (_) {
      // Persistence is best-effort; in-memory token still works.
    }
  }
}
