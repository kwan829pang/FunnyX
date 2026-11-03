import 'dart:async';

import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Separate session for Corp Web wizards (Bearer `corp_…`).
class CorpSessionController extends GetxController implements TokenStore {
  static const _kAccess = 'fx_corp_access';

  final RxnString _token = RxnString();
  final Rxn<CorpProfile> profile = Rxn<CorpProfile>();
  final RxnString lastApiSecret = RxnString();
  SharedPreferences? _prefs;

  @override
  String? get token => _token.value;

  @override
  bool get isAuthenticated {
    final value = _token.value;
    return value != null && value.isNotEmpty;
  }

  Future<void> restore() async {
    _prefs = await SharedPreferences.getInstance();
    final access = _prefs!.getString(_kAccess);
    if (access != null && access.isNotEmpty) {
      _token.value = access;
    }
  }

  @override
  void setToken(String? value) {
    final access = value?.trim();
    _token.value = (access != null && access.isNotEmpty) ? access : null;
    if (_token.value == null) {
      profile.value = null;
      lastApiSecret.value = null;
    }
    unawaited(_persist());
  }

  void setAuth(CorpAuthResult result) {
    _token.value = result.accessToken;
    profile.value = result.profile;
    if (result.apiSecret != null && result.apiSecret!.isNotEmpty) {
      lastApiSecret.value = result.apiSecret;
    }
    unawaited(_persist());
  }

  void clear() {
    _token.value = null;
    profile.value = null;
    lastApiSecret.value = null;
    unawaited(_persist());
  }

  Future<void> _persist() async {
    try {
      final prefs = _prefs ?? await SharedPreferences.getInstance();
      _prefs = prefs;
      final access = _token.value;
      if (access == null || access.isEmpty) {
        await prefs.remove(_kAccess);
      } else {
        await prefs.setString(_kAccess, access);
      }
    } catch (_) {}
  }
}
