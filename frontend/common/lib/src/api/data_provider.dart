import 'package:dio/dio.dart';

import '../models/api_error.dart';
import '../models/session_token.dart';
import 'api_client.dart';

/// Swappable data layer. UI/GetX should depend on this, not Dio types.
abstract class AuthDataProvider {
  Future<SessionToken> login({
    required String username,
    required String password,
  });

  Future<SessionToken> register({
    required String username,
    required String password,
  });

  Future<void> logout({String? accessToken, String? refreshToken});
}

/// Local mock used for offline UI tests.
class MockAuthDataProvider implements AuthDataProvider {
  MockAuthDataProvider({this.role = 'user'});

  final String role;

  @override
  Future<SessionToken> login({
    required String username,
    required String password,
  }) async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    if (username.isEmpty || password.isEmpty) {
      throw ArgumentError('username and password required');
    }
    return SessionToken(
      token: 'mock.$role.${username.hashCode}',
      accountId: username,
      username: username,
    );
  }

  @override
  Future<SessionToken> register({
    required String username,
    required String password,
  }) =>
      login(username: username, password: password);

  @override
  Future<void> logout({String? accessToken, String? refreshToken}) async {}
}

/// Client Web auth against Client Center (`/v1/client/login|register|logout`).
///
/// Tokens are issued by Session Token Server via Client Center.
class ClientCenterAuthProvider implements AuthDataProvider {
  ClientCenterAuthProvider(this.api);

  final ApiClient api;

  static const _loginPath = '/v1/client/login';
  static const _registerPath = '/v1/client/register';
  static const _logoutPath = '/v1/client/logout';

  @override
  Future<SessionToken> login({
    required String username,
    required String password,
  }) {
    return _postAuth(_loginPath, username: username, password: password);
  }

  @override
  Future<SessionToken> register({
    required String username,
    required String password,
  }) {
    return _postAuth(_registerPath, username: username, password: password);
  }

  @override
  Future<void> logout({String? accessToken, String? refreshToken}) async {
    try {
      await api.dio.post<Map<String, dynamic>>(
        _logoutPath,
        data: <String, dynamic>{
          if (accessToken != null && accessToken.isNotEmpty)
            'access_token': accessToken,
          if (refreshToken != null && refreshToken.isNotEmpty)
            'refresh_token': refreshToken,
        },
        options: Options(
          extra: {'skipAuth': accessToken == null || accessToken.isEmpty},
          headers: accessToken != null && accessToken.isNotEmpty
              ? {'Authorization': 'Bearer $accessToken'}
              : null,
        ),
      );
    } on DioException catch (e) {
      // Best-effort revoke; local clear still proceeds.
      if (e.response?.statusCode == 401) {
        return;
      }
      throw _mapDio(e);
    }
  }

  Future<SessionToken> _postAuth(
    String path, {
    required String username,
    required String password,
  }) async {
    if (username.trim().isEmpty || password.isEmpty) {
      throw ArgumentError('username and password required');
    }
    try {
      final response = await api.dio.post<Map<String, dynamic>>(
        path,
        data: <String, dynamic>{
          'username': username.trim(),
          'password': password,
        },
        options: Options(extra: const {'skipAuth': true}),
      );
      final data = response.data;
      if (data == null) {
        throw const ApiError('empty auth response');
      }
      return SessionToken.fromAuthJson(data);
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  ApiError _mapDio(DioException e) {
    final status = e.response?.statusCode;
    final body = e.response?.data;
    String message = e.message ?? 'request failed';
    if (body is Map && body['error'] != null) {
      message = body['error'].toString();
    }
    return ApiError(message, statusCode: status);
  }
}

/// Admin Panel auth against Admin API (`/v1/admin/login|logout`).
///
/// Tokens are issued by Session Token Server (`actor_type=admin`) via Admin API.
class AdminApiAuthProvider implements AuthDataProvider {
  AdminApiAuthProvider(this.api);

  final ApiClient api;

  static const _loginPath = '/v1/admin/login';
  static const _logoutPath = '/v1/admin/logout';

  @override
  Future<SessionToken> login({
    required String username,
    required String password,
  }) async {
    if (username.trim().isEmpty || password.isEmpty) {
      throw ArgumentError('username and password required');
    }
    try {
      final response = await api.dio.post<Map<String, dynamic>>(
        _loginPath,
        data: <String, dynamic>{
          'username': username.trim(),
          'password': password,
        },
        options: Options(extra: const {'skipAuth': true}),
      );
      final data = response.data;
      if (data == null) {
        throw const ApiError('empty auth response');
      }
      return SessionToken.fromAuthJson(data);
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<SessionToken> register({
    required String username,
    required String password,
  }) {
    throw UnsupportedError('Admin Panel does not support self-register');
  }

  @override
  Future<void> logout({String? accessToken, String? refreshToken}) async {
    try {
      await api.dio.post<Map<String, dynamic>>(
        _logoutPath,
        data: <String, dynamic>{
          if (accessToken != null && accessToken.isNotEmpty)
            'access_token': accessToken,
          if (refreshToken != null && refreshToken.isNotEmpty)
            'refresh_token': refreshToken,
        },
        options: Options(
          extra: {'skipAuth': accessToken == null || accessToken.isEmpty},
          headers: accessToken != null && accessToken.isNotEmpty
              ? {'Authorization': 'Bearer $accessToken'}
              : null,
        ),
      );
    } on DioException catch (e) {
      if (e.response?.statusCode == 401) {
        return;
      }
      throw _mapDio(e);
    }
  }

  ApiError _mapDio(DioException e) {
    final status = e.response?.statusCode;
    final body = e.response?.data;
    String message = e.message ?? 'request failed';
    if (body is Map && body['error'] != null) {
      message = body['error'].toString();
    }
    return ApiError(message, statusCode: status);
  }
}
