import 'package:dio/dio.dart';
import 'package:http_parser/http_parser.dart';

import '../models/api_error.dart';
import '../models/corp_models.dart';
import 'api_client.dart';

abstract class CorpDataProvider {
  Future<CorpAuthResult> register({
    required String companyName,
    required String contactName,
    required String contactEmail,
    required String password,
    required List<int> brBytes,
    required String brFileName,
    String brContentType = 'application/octet-stream',
  });

  Future<CorpAuthResult> login({
    required String login,
    required String password,
  });

  Future<CorpProfile> profile();

  Future<List<CorpGame>> listGames();

  Future<CorpGame> submitGame({
    required String gameName,
    required String gameCode,
    String? partnerCode,
    String? partnerGameId,
    String? description,
    required List<int>? governmentBytes,
    String? governmentFileName,
    required List<int>? materialBytes,
    String? materialFileName,
  });
}

class ClientCenterCorpProvider implements CorpDataProvider {
  ClientCenterCorpProvider(this.api);

  final ApiClient api;

  @override
  Future<CorpAuthResult> register({
    required String companyName,
    required String contactName,
    required String contactEmail,
    required String password,
    required List<int> brBytes,
    required String brFileName,
    String brContentType = 'application/octet-stream',
  }) async {
    try {
      final form = FormData.fromMap({
        'company_name': companyName,
        'contact_name': contactName,
        'contact_email': contactEmail,
        'password': password,
        'br_file': MultipartFile.fromBytes(
          brBytes,
          filename: brFileName,
          contentType: MediaType.parse(brContentType),
        ),
      });
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/corp/register',
        data: form,
        options: Options(extra: {'skipAuth': true}),
      );
      return _authFrom(res.data);
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<CorpAuthResult> login({
    required String login,
    required String password,
  }) async {
    try {
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/corp/login',
        data: {'login': login, 'password': password},
        options: Options(extra: {'skipAuth': true}),
      );
      return _authFrom(res.data);
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<CorpProfile> profile() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>('/v1/corp/profile');
      final p = res.data?['profile'];
      if (p is! Map) throw const ApiError('invalid profile');
      return CorpProfile.fromJson(Map<String, dynamic>.from(p));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<List<CorpGame>> listGames() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>('/v1/corp/games');
      final list = res.data?['games'];
      if (list is! List) return const [];
      return list
          .whereType<Map>()
          .map((e) => CorpGame.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<CorpGame> submitGame({
    required String gameName,
    required String gameCode,
    String? partnerCode,
    String? partnerGameId,
    String? description,
    required List<int>? governmentBytes,
    String? governmentFileName,
    required List<int>? materialBytes,
    String? materialFileName,
  }) async {
    try {
      final map = <String, dynamic>{
        'game_name': gameName,
        'game_code': gameCode,
        if (partnerCode != null && partnerCode.isNotEmpty)
          'partner_code': partnerCode,
        if (partnerGameId != null && partnerGameId.isNotEmpty)
          'partner_game_id': partnerGameId,
        if (description != null) 'description': description,
      };
      if (governmentBytes != null && governmentBytes.isNotEmpty) {
        map['government_approval'] = MultipartFile.fromBytes(
          governmentBytes,
          filename: governmentFileName ?? 'government.pdf',
        );
      }
      if (materialBytes != null && materialBytes.isNotEmpty) {
        map['approved_material'] = MultipartFile.fromBytes(
          materialBytes,
          filename: materialFileName ?? 'material.pdf',
        );
      }
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/corp/games',
        data: FormData.fromMap(map),
      );
      final g = res.data?['game'] ?? res.data;
      if (g is! Map) throw const ApiError('invalid game response');
      return CorpGame.fromJson(Map<String, dynamic>.from(g));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  CorpAuthResult _authFrom(Map<String, dynamic>? data) {
    if (data == null) throw const ApiError('empty response');
    final p = data['profile'];
    if (p is! Map) throw const ApiError('missing profile');
    return CorpAuthResult(
      profile: CorpProfile.fromJson(Map<String, dynamic>.from(p)),
      accessToken: data['access_token']?.toString() ?? '',
      expiresAt: (data['expires_at'] as num?)?.toInt() ?? 0,
      apiKey: data['api_key']?.toString(),
      apiSecret: data['api_secret']?.toString(),
      message: data['message']?.toString(),
    );
  }

  ApiError _mapDio(DioException e) {
    final data = e.response?.data;
    if (data is Map && data['error'] != null) {
      return ApiError(data['error'].toString(), statusCode: e.response?.statusCode);
    }
    return ApiError(e.message ?? 'request failed', statusCode: e.response?.statusCode);
  }
}
