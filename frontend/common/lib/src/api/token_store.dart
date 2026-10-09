/// Holds the Session Token Server JWT used on authenticated API calls.
abstract class TokenStore {
  String? get token;
  void setToken(String? value);
  bool get isAuthenticated;
}

/// In-memory token store (swap for secure storage later).
class MemoryTokenStore implements TokenStore {
  String? _token;

  @override
  String? get token => _token;

  @override
  bool get isAuthenticated {
    final value = _token;
    return value != null && value.isNotEmpty;
  }

  @override
  void setToken(String? value) {
    _token = value;
  }
}
