import 'dart:convert';

import 'package:crypto/crypto.dart';

/// Client-side password transport digest (SHA-256 hex).
///
/// Flutter clients hash the UTF-8 password before POST so the wire body never
/// carries plaintext. Server stores Argon2id of this digest.
abstract final class PasswordDigest {
  /// Lowercase hex SHA-256 of [password] UTF-8 bytes.
  static String sha256Hex(String password) {
    final digest = sha256.convert(utf8.encode(password));
    return digest.toString();
  }
}
