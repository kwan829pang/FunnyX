import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

/// Separate session for Corp Web wizards (Bearer `corp_…`).
class CorpSessionController extends GetxController implements TokenStore {
  final RxnString _token = RxnString();
  final Rxn<CorpProfile> profile = Rxn<CorpProfile>();
  final RxnString lastApiSecret = RxnString();

  @override
  String? get token => _token.value;

  @override
  bool get isAuthenticated {
    final value = _token.value;
    return value != null && value.isNotEmpty;
  }

  @override
  void setToken(String? value) {
    _token.value = value;
  }

  void setAuth(CorpAuthResult result) {
    _token.value = result.accessToken;
    profile.value = result.profile;
    if (result.apiSecret != null && result.apiSecret!.isNotEmpty) {
      lastApiSecret.value = result.apiSecret;
    }
  }

  void clear() {
    _token.value = null;
    profile.value = null;
    lastApiSecret.value = null;
  }
}
