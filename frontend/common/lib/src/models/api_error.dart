class ApiError implements Exception {
  const ApiError(this.message, {this.statusCode});

  final String message;
  final int? statusCode;

  bool get isUnauthorized {
    if (statusCode == 401) return true;
    final m = message.toLowerCase();
    return m.contains('invalid_or_expired') ||
        m.contains('invalid or expired') ||
        m.contains('invalid session') ||
        m.contains('missing session token') ||
        m.contains('missing bearer');
  }

  @override
  String toString() => 'ApiError($statusCode): $message';
}
