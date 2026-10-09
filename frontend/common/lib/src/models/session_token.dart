class SessionToken {
  const SessionToken({
    required this.token,
    required this.accountId,
    this.refreshToken,
    this.endUserId,
    this.expiresAtMs,
    this.username,
  });

  final String token;
  final String accountId;
  final String? refreshToken;
  final int? endUserId;
  final int? expiresAtMs;
  final String? username;

  /// Parse Client Center / Session Token Server login|register JSON.
  factory SessionToken.fromAuthJson(Map<String, dynamic> json) {
    final access = json['access_token']?.toString();
    if (access == null || access.isEmpty) {
      throw const FormatException('access_token missing in auth response');
    }
    final account = json['account_id']?.toString() ??
        json['username']?.toString() ??
        '';
    return SessionToken(
      token: access,
      refreshToken: json['refresh_token']?.toString(),
      accountId: account,
      endUserId: _asInt(json['end_user_id']) ?? _asInt(json['admin_user_id']),
      expiresAtMs: _asInt(json['expires_at_ms']),
      username: json['username']?.toString(),
    );
  }

  static int? _asInt(Object? value) {
    if (value is int) {
      return value;
    }
    if (value is num) {
      return value.toInt();
    }
    return int.tryParse(value?.toString() ?? '');
  }
}
