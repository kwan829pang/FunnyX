import 'package:dio/dio.dart';

import 'token_store.dart';

/// Dio client that attaches `Authorization: Bearer <token>` except public paths.
class ApiClient {
  ApiClient({
    required String baseUrl,
    required this.tokenStore,
    this.onUnauthorized,
    Iterable<String> publicPathPrefixes = const [
      '/v1/client/login',
      '/v1/client/register',
      '/v1/client/oauth',
      '/v1/admin/login',
      '/v1/corp/login',
      '/v1/corp/register',
      '/v1/oauth',
      '/health',
    ],
    Duration timeout = const Duration(seconds: 20),
  }) : _publicPathPrefixes = publicPathPrefixes.toList(growable: false),
       dio = Dio(
         BaseOptions(
           baseUrl: baseUrl,
           connectTimeout: timeout,
           receiveTimeout: timeout,
           headers: const {'Accept': 'application/json'},
         ),
       ) {
    dio.interceptors.add(
      InterceptorsWrapper(
        onRequest: _onRequest,
        onError: _onError,
      ),
    );
  }

  final TokenStore tokenStore;
  final List<String> _publicPathPrefixes;
  final Dio dio;

  /// Invoked when a protected call has no token or the server returns 401.
  final void Function()? onUnauthorized;

  void _onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    final skipAuth =
        options.extra['skipAuth'] == true || _isPublic(_requestPath(options));
    if (!skipAuth) {
      final token = tokenStore.token;
      if (token == null || token.isEmpty) {
        onUnauthorized?.call();
        handler.reject(
          DioException(
            requestOptions: options,
            type: DioExceptionType.badResponse,
            error: 'unauthenticated',
            response: Response(
              requestOptions: options,
              statusCode: 401,
              statusMessage: 'Unauthorized',
              data: <String, dynamic>{
                'error': 'missing session token — login required',
              },
            ),
          ),
        );
        return;
      }
      options.headers['Authorization'] = 'Bearer $token';
    }
    handler.next(options);
  }

  void _onError(DioException err, ErrorInterceptorHandler handler) {
    final skipAuth = err.requestOptions.extra['skipAuth'] == true ||
        _isPublic(_requestPath(err.requestOptions));
    if (!skipAuth && _isAuthExpired(err)) {
      onUnauthorized?.call();
    }
    handler.next(err);
  }

  /// 401 or STS/Gateway body like `{ "error": "invalid_or_expired" }`.
  static bool _isAuthExpired(DioException err) {
    if (err.response?.statusCode == 401) {
      return true;
    }
    final data = err.response?.data;
    if (data is Map) {
      for (final key in ['error', 'reason', 'message']) {
        final value = data[key]?.toString().toLowerCase() ?? '';
        if (value.contains('invalid_or_expired') ||
            value.contains('invalid or expired') ||
            value.contains('invalid session')) {
          return true;
        }
      }
    }
    return false;
  }

  /// Path only (no host/query), always leading `/`.
  static String _requestPath(RequestOptions options) {
    var path = options.path;
    if (path.startsWith('http://') || path.startsWith('https://')) {
      path = Uri.parse(path).path;
    }
    final q = path.indexOf('?');
    if (q >= 0) {
      path = path.substring(0, q);
    }
    if (path.isEmpty) {
      return '/';
    }
    return path.startsWith('/') ? path : '/$path';
  }

  bool _isPublic(String path) {
    for (final prefix in _publicPathPrefixes) {
      if (path == prefix || path.startsWith('$prefix/')) {
        return true;
      }
    }
    return false;
  }
}
