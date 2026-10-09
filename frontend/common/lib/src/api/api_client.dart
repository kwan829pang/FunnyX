import 'package:dio/dio.dart';

import 'token_store.dart';

/// Dio client that attaches `Authorization: Bearer <jwt>` except public paths.
class ApiClient {
  ApiClient({
    required String baseUrl,
    required this.tokenStore,
    Iterable<String> publicPathPrefixes = const [
      '/v1/client/login',
      '/v1/client/register',
      '/v1/client/oauth',
      '/v1/admin/login',
      '/v1/oauth',
      '/login',
      '/register',
      '/oauth',
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
      InterceptorsWrapper(onRequest: _onRequest),
    );
  }

  final TokenStore tokenStore;
  final List<String> _publicPathPrefixes;
  final Dio dio;

  void _onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    final skipAuth = options.extra['skipAuth'] == true || _isPublic(options.path);
    if (!skipAuth) {
      final token = tokenStore.token;
      if (token == null || token.isEmpty) {
        handler.reject(
          DioException(
            requestOptions: options,
            type: DioExceptionType.badResponse,
            error: 'unauthenticated',
            response: Response(requestOptions: options, statusCode: 401),
          ),
        );
        return;
      }
      options.headers['Authorization'] = 'Bearer $token';
    }
    handler.next(options);
  }

  bool _isPublic(String path) {
    return _publicPathPrefixes.any(path.contains);
  }
}
