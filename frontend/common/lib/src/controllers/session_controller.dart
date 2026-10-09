import 'package:get/get.dart';

import '../api/token_store.dart';

/// GetX session state: JWT from Session Token Server.
class SessionController extends GetxController implements TokenStore {
  final RxnString _token = RxnString();

  @override
  String? get token => _token.value;

  RxnString get tokenRx => _token;

  @override
  bool get isAuthenticated {
    final value = _token.value;
    return value != null && value.isNotEmpty;
  }

  @override
  void setToken(String? value) {
    _token.value = value;
  }

  void clear() => setToken(null);
}
