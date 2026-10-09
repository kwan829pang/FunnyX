import 'package:dio/dio.dart';

import '../models/api_error.dart';
import '../models/shop_models.dart';
import 'api_client.dart';

/// Client Center e-shop + game-account bindings for checkout.
abstract class ShopDataProvider {
  Future<String> baseCurrency();

  Future<List<ShopPackage>> listPackages();

  Future<List<CorpProduct>> listCorpProducts();

  Future<ShopOrder> createOrder({
    required String sellerType,
    int? packageId,
    int? corpProductId,
    required int gameAccountId,
    String? returnUrl,
  });

  Future<List<ShopOrder>> listOrders();

  Future<ShopOrder> getOrder(int id);

  Future<ShopOrder> cancelOrder(int id);

  Future<List<GameAccountBinding>> listGameAccounts();
}

class ClientCenterShopProvider implements ShopDataProvider {
  ClientCenterShopProvider(this.api);

  final ApiClient api;

  @override
  Future<String> baseCurrency() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>(
        '/v1/shop/base-currency',
      );
      return res.data?['base_fiat_currency']?.toString() ?? 'HKD';
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<List<ShopPackage>> listPackages() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>('/v1/shop/packages');
      final list = res.data?['packages'];
      if (list is! List) return const [];
      return list
          .whereType<Map>()
          .map((e) => ShopPackage.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<List<CorpProduct>> listCorpProducts() async {
    try {
      final res =
          await api.dio.get<Map<String, dynamic>>('/v1/shop/corp-products');
      final list = res.data?['products'];
      if (list is! List) return const [];
      return list
          .whereType<Map>()
          .map((e) => CorpProduct.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<ShopOrder> createOrder({
    required String sellerType,
    int? packageId,
    int? corpProductId,
    required int gameAccountId,
    String? returnUrl,
  }) async {
    try {
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/shop/orders',
        data: <String, dynamic>{
          'seller_type': sellerType,
          if (packageId != null) 'package_id': packageId,
          if (corpProductId != null) 'corp_product_id': corpProductId,
          'game_account_id': gameAccountId,
          if (returnUrl != null && returnUrl.isNotEmpty)
            'return_url': returnUrl,
        },
      );
      final data = res.data;
      if (data == null) throw const ApiError('empty create order response');
      return ShopOrder.fromJson(data);
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<List<ShopOrder>> listOrders() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>('/v1/shop/orders');
      final list = res.data?['orders'];
      if (list is! List) return const [];
      return list
          .whereType<Map>()
          .map((e) => ShopOrder.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<ShopOrder> getOrder(int id) async {
    try {
      final res =
          await api.dio.get<Map<String, dynamic>>('/v1/shop/orders/$id');
      final data = res.data;
      if (data == null) throw const ApiError('empty order response');
      // get_order may wrap or return order directly
      if (data['order'] is Map) {
        return ShopOrder.fromJson(
          Map<String, dynamic>.from(data['order'] as Map),
        );
      }
      return ShopOrder.fromJson(data);
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<ShopOrder> cancelOrder(int id) async {
    try {
      final res =
          await api.dio.post<Map<String, dynamic>>('/v1/shop/orders/$id/cancel');
      final data = res.data;
      if (data == null) throw const ApiError('empty cancel response');
      if (data['order'] is Map) {
        return ShopOrder.fromJson(
          Map<String, dynamic>.from(data['order'] as Map),
        );
      }
      return ShopOrder.fromJson(data);
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<List<GameAccountBinding>> listGameAccounts() async {
    try {
      final res = await api.dio.get<dynamic>('/v1/client/game-accounts');
      final data = res.data;
      if (data is List) {
        return data
            .whereType<Map>()
            .map(
              (e) => GameAccountBinding.fromJson(Map<String, dynamic>.from(e)),
            )
            .where((b) => b.status == 'active')
            .toList();
      }
      if (data is Map && data['bindings'] is List) {
        return (data['bindings'] as List)
            .whereType<Map>()
            .map(
              (e) => GameAccountBinding.fromJson(Map<String, dynamic>.from(e)),
            )
            .where((b) => b.status == 'active')
            .toList();
      }
      return const [];
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
