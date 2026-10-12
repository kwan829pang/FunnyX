import 'package:dio/dio.dart';

import '../models/api_error.dart';
import 'api_client.dart';

class ExchangeMarket {
  const ExchangeMarket({
    required this.id,
    required this.symbol,
    required this.marketName,
    required this.status,
  });

  final int id;
  final String symbol;
  final String marketName;
  final String status;

  factory ExchangeMarket.fromJson(Map<String, dynamic> json) {
    return ExchangeMarket(
      id: (json['id'] as num?)?.toInt() ?? 0,
      symbol: json['symbol']?.toString() ?? '',
      marketName: json['market_name']?.toString() ?? '',
      status: json['status']?.toString() ?? '',
    );
  }
}

class ExchangeOrder {
  const ExchangeOrder({
    required this.id,
    required this.symbol,
    required this.side,
    required this.orderType,
    required this.price,
    required this.quantity,
    required this.remaining,
    required this.status,
    required this.engineOrderId,
  });

  final int id;
  final String symbol;
  final String side;
  final String orderType;
  final int price;
  final int quantity;
  final int remaining;
  final String status;
  final String engineOrderId;

  factory ExchangeOrder.fromJson(Map<String, dynamic> json) {
    return ExchangeOrder(
      id: (json['id'] as num?)?.toInt() ?? 0,
      symbol: json['symbol']?.toString() ?? '',
      side: json['side']?.toString() ?? '',
      orderType: json['order_type']?.toString() ?? '',
      price: (json['price'] as num?)?.toInt() ?? 0,
      quantity: (json['quantity'] as num?)?.toInt() ?? 0,
      remaining: (json['remaining'] as num?)?.toInt() ?? 0,
      status: json['status']?.toString() ?? '',
      engineOrderId: json['engine_order_id']?.toString() ?? '',
    );
  }
}

abstract class ExchangeDataProvider {
  Future<List<ExchangeMarket>> listMarkets();

  Future<Map<String, dynamic>> getBook(int marketId, {int levels = 10});

  Future<List<ExchangeOrder>> listOrders();

  Future<ExchangeOrder> placeOrder({
    required String symbol,
    required String side,
    required String orderType,
    required int quantity,
    int price = 0,
    required int gameAccountId,
    int? marketPairId,
  });

  Future<ExchangeOrder> cancelOrder(int id);
}

class ClientCenterExchangeProvider implements ExchangeDataProvider {
  ClientCenterExchangeProvider(this.api);

  final ApiClient api;

  @override
  Future<List<ExchangeMarket>> listMarkets() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>('/v1/markets');
      final raw = res.data?['markets'];
      if (raw is! List) return const [];
      return raw
          .whereType<Map>()
          .map((e) => ExchangeMarket.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<Map<String, dynamic>> getBook(int marketId, {int levels = 10}) async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>(
        '/v1/markets/$marketId/book',
        queryParameters: {'levels': levels},
      );
      return Map<String, dynamic>.from(res.data ?? const {});
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<List<ExchangeOrder>> listOrders() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>('/v1/orders');
      final raw = res.data?['orders'];
      if (raw is! List) return const [];
      return raw
          .whereType<Map>()
          .map((e) => ExchangeOrder.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<ExchangeOrder> placeOrder({
    required String symbol,
    required String side,
    required String orderType,
    required int quantity,
    int price = 0,
    required int gameAccountId,
    int? marketPairId,
  }) async {
    try {
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/orders',
        data: <String, dynamic>{
          'symbol': symbol,
          'side': side,
          'order_type': orderType,
          'quantity': quantity,
          'price': price,
          'game_account_id': gameAccountId,
          if (marketPairId != null) 'market_pair_id': marketPairId,
        },
      );
      final order = res.data?['order'];
      if (order is! Map) throw const ApiError('invalid order response');
      return ExchangeOrder.fromJson(Map<String, dynamic>.from(order));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<ExchangeOrder> cancelOrder(int id) async {
    try {
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/orders/$id/cancel',
      );
      final order = res.data?['order'];
      if (order is! Map) throw const ApiError('invalid order response');
      return ExchangeOrder.fromJson(Map<String, dynamic>.from(order));
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
