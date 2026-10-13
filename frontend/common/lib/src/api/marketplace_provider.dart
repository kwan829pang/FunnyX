import 'package:dio/dio.dart';

import '../models/api_error.dart';
import '../models/marketplace_models.dart';
import 'api_client.dart';

abstract class MarketplaceDataProvider {
  Future<MarketplaceEligibility> eligibility();

  Future<List<MarketplaceItem>> listItems({required int gameAccountId});

  Future<List<MarketplaceDeal>> listDeals({int limit = 50});

  Future<MarketplaceDeal> getDeal(int id);

  Future<MarketplaceDeal> placeDeal({
    required int posterGameAccountId,
    required String offerAssetType,
    int? offerGameCoinId,
    String? offerItemRefId,
    required double offerAmount,
    required String wantAssetType,
    required int wantGameCoinId,
    required double wantAmount,
  });

  Future<MarketplaceDeal> takeDeal({
    required int dealId,
    required int requesterGameAccountId,
  });

  Future<MarketplaceDeal> cancelDeal(int dealId);
}

class ClientCenterMarketplaceProvider implements MarketplaceDataProvider {
  ClientCenterMarketplaceProvider(this.api);

  final ApiClient api;

  @override
  Future<MarketplaceEligibility> eligibility() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>(
        '/v1/marketplace/eligibility',
      );
      final elig = res.data?['eligibility'];
      if (elig is! Map) {
        throw const ApiError('invalid eligibility response');
      }
      return MarketplaceEligibility.fromJson(Map<String, dynamic>.from(elig));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<List<MarketplaceItem>> listItems({required int gameAccountId}) async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>(
        '/v1/marketplace/items',
        queryParameters: {'game_account_id': gameAccountId},
      );
      final list = res.data?['items'];
      if (list is! List) return const [];
      return list
          .whereType<Map>()
          .map((e) => MarketplaceItem.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<List<MarketplaceDeal>> listDeals({int limit = 50}) async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>(
        '/v1/marketplace/deals',
        queryParameters: {'limit': limit},
      );
      final list = res.data?['deals'];
      if (list is! List) return const [];
      return list
          .whereType<Map>()
          .map((e) => MarketplaceDeal.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<MarketplaceDeal> getDeal(int id) async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>(
        '/v1/marketplace/deals/$id',
      );
      final data = res.data;
      if (data == null) throw const ApiError('deal not found');
      return MarketplaceDeal.fromJson(Map<String, dynamic>.from(data));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<MarketplaceDeal> placeDeal({
    required int posterGameAccountId,
    required String offerAssetType,
    int? offerGameCoinId,
    String? offerItemRefId,
    required double offerAmount,
    required String wantAssetType,
    required int wantGameCoinId,
    required double wantAmount,
  }) async {
    try {
      final body = <String, dynamic>{
        'poster_game_account_id': posterGameAccountId,
        'offer_asset_type': offerAssetType,
        'offer_amount': offerAmount,
        'want_asset_type': wantAssetType,
        'want_game_coin_id': wantGameCoinId,
        'want_amount': wantAmount,
      };
      if (offerGameCoinId != null) {
        body['offer_game_coin_id'] = offerGameCoinId;
      }
      if (offerItemRefId != null && offerItemRefId.isNotEmpty) {
        body['offer_item_ref_id'] = offerItemRefId;
      }
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/marketplace/deals',
        data: body,
      );
      final data = res.data;
      if (data == null) throw const ApiError('invalid place response');
      return MarketplaceDeal.fromJson(Map<String, dynamic>.from(data));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<MarketplaceDeal> takeDeal({
    required int dealId,
    required int requesterGameAccountId,
  }) async {
    try {
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/marketplace/deals/$dealId/request',
        data: {'requester_game_account_id': requesterGameAccountId},
      );
      final data = res.data;
      if (data == null) throw const ApiError('invalid take response');
      return MarketplaceDeal.fromJson(Map<String, dynamic>.from(data));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<MarketplaceDeal> cancelDeal(int dealId) async {
    try {
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/marketplace/deals/$dealId/cancel',
      );
      final data = res.data;
      if (data == null) throw const ApiError('invalid cancel response');
      return MarketplaceDeal.fromJson(Map<String, dynamic>.from(data));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  ApiError _mapDio(DioException e) {
    final data = e.response?.data;
    if (data is Map && data['error'] != null) {
      return ApiError(data['error'].toString(), statusCode: e.response?.statusCode);
    }
    return ApiError(e.message ?? 'request failed', statusCode: e.response?.statusCode);
  }
}
