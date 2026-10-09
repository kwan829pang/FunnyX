import 'package:dio/dio.dart';

import '../models/api_error.dart';
import 'api_client.dart';

class AdminRequiredItem {
  const AdminRequiredItem({
    required this.key,
    required this.label,
    required this.ready,
    required this.detail,
  });

  final String key;
  final String label;
  final bool ready;
  final String detail;

  factory AdminRequiredItem.fromJson(Map<String, dynamic> json) {
    return AdminRequiredItem(
      key: json['key']?.toString() ?? '',
      label: json['label']?.toString() ?? '',
      ready: json['ready'] == true,
      detail: json['detail']?.toString() ?? '',
    );
  }
}

class AdminShopPackage {
  const AdminShopPackage({
    required this.id,
    required this.code,
    required this.name,
    required this.coinAmount,
    required this.fiatPrice,
    required this.status,
  });

  final int id;
  final String code;
  final String name;
  final double coinAmount;
  final double fiatPrice;
  final String status;

  factory AdminShopPackage.fromJson(Map<String, dynamic> json) {
    return AdminShopPackage(
      id: (json['id'] as num?)?.toInt() ?? 0,
      code: json['code']?.toString() ?? '',
      name: json['name']?.toString() ?? '',
      coinAmount: (json['coin_amount'] as num?)?.toDouble() ?? 0,
      fiatPrice: (json['fiat_price'] as num?)?.toDouble() ?? 0,
      status: json['status']?.toString() ?? '',
    );
  }
}

class AdminSetupStatus {
  const AdminSetupStatus({
    required this.initialized,
    required this.baseFiatCurrency,
    required this.baseFiatReady,
    required this.packagesReady,
    required this.activePackageCount,
    required this.required,
    required this.packages,
  });

  final bool initialized;
  final String? baseFiatCurrency;
  final bool baseFiatReady;
  final bool packagesReady;
  final int activePackageCount;
  final List<AdminRequiredItem> required;
  final List<AdminShopPackage> packages;

  factory AdminSetupStatus.fromJson(Map<String, dynamic> json) {
    final requiredRaw = json['required'];
    final packagesRaw = json['packages'];
    return AdminSetupStatus(
      initialized: json['initialized'] == true,
      baseFiatCurrency: json['base_fiat_currency']?.toString(),
      baseFiatReady: json['base_fiat_ready'] == true,
      packagesReady: json['packages_ready'] == true,
      activePackageCount: (json['active_package_count'] as num?)?.toInt() ?? 0,
      required: requiredRaw is List
          ? requiredRaw
              .whereType<Map>()
              .map((e) => AdminRequiredItem.fromJson(Map<String, dynamic>.from(e)))
              .toList()
          : const [],
      packages: packagesRaw is List
          ? packagesRaw
              .whereType<Map>()
              .map((e) => AdminShopPackage.fromJson(Map<String, dynamic>.from(e)))
              .toList()
          : const [],
    );
  }
}

class AdminCorpProduct {
  const AdminCorpProduct({
    required this.id,
    required this.corporateUserId,
    required this.code,
    required this.name,
    required this.productType,
    required this.fiatPrice,
    required this.status,
    this.gameId,
    this.creditGameCoinId,
    this.creditGameCoin,
    this.creditAmount,
    this.itemCode,
  });

  final int id;
  final int corporateUserId;
  final int? gameId;
  final String code;
  final String name;
  final String productType;
  final int? creditGameCoinId;
  final String? creditGameCoin;
  final double? creditAmount;
  final String? itemCode;
  final double fiatPrice;
  final String status;

  factory AdminCorpProduct.fromJson(Map<String, dynamic> json) {
    return AdminCorpProduct(
      id: (json['id'] as num?)?.toInt() ?? 0,
      corporateUserId: (json['corporate_user_id'] as num?)?.toInt() ?? 0,
      gameId: (json['game_id'] as num?)?.toInt(),
      code: json['code']?.toString() ?? '',
      name: json['name']?.toString() ?? '',
      productType: json['product_type']?.toString() ?? '',
      creditGameCoinId: (json['credit_game_coin_id'] as num?)?.toInt(),
      creditGameCoin: json['credit_game_coin']?.toString(),
      creditAmount: (json['credit_amount'] as num?)?.toDouble(),
      itemCode: json['item_code']?.toString(),
      fiatPrice: (json['fiat_price'] as num?)?.toDouble() ?? 0,
      status: json['status']?.toString() ?? '',
    );
  }
}

class AdminMarketPool {
  const AdminMarketPool({
    required this.id,
    required this.poolDepth,
    required this.initialPrice,
    required this.baseAmount,
    required this.quoteAmount,
    required this.status,
  });

  final int id;
  final double poolDepth;
  final double initialPrice;
  final double baseAmount;
  final double quoteAmount;
  final String status;

  factory AdminMarketPool.fromJson(Map<String, dynamic> json) {
    return AdminMarketPool(
      id: (json['id'] as num?)?.toInt() ?? 0,
      poolDepth: (json['pool_depth'] as num?)?.toDouble() ?? 0,
      initialPrice: (json['initial_price'] as num?)?.toDouble() ?? 0,
      baseAmount: (json['base_amount'] as num?)?.toDouble() ?? 0,
      quoteAmount: (json['quote_amount'] as num?)?.toDouble() ?? 0,
      status: json['status']?.toString() ?? '',
    );
  }
}

class AdminMarket {
  const AdminMarket({
    required this.id,
    required this.corporateUserId,
    required this.gameId,
    required this.marketName,
    required this.fundingSource,
    required this.status,
    this.baseGameCoin,
    this.quoteGameCoin,
    this.lockAmount,
    this.pool,
  });

  final int id;
  final int corporateUserId;
  final int gameId;
  final String marketName;
  final String fundingSource;
  final String status;
  final String? baseGameCoin;
  final String? quoteGameCoin;
  final double? lockAmount;
  final AdminMarketPool? pool;

  factory AdminMarket.fromJson(Map<String, dynamic> json) {
    final poolRaw = json['pool'];
    return AdminMarket(
      id: (json['id'] as num?)?.toInt() ?? 0,
      corporateUserId: (json['corporate_user_id'] as num?)?.toInt() ?? 0,
      gameId: (json['game_id'] as num?)?.toInt() ?? 0,
      marketName: json['market_name']?.toString() ?? '',
      fundingSource: json['funding_source']?.toString() ?? '',
      status: json['status']?.toString() ?? '',
      baseGameCoin: json['base_game_coin']?.toString(),
      quoteGameCoin: json['quote_game_coin']?.toString(),
      lockAmount: (json['lock_amount'] as num?)?.toDouble(),
      pool: poolRaw is Map
          ? AdminMarketPool.fromJson(Map<String, dynamic>.from(poolRaw))
          : null,
    );
  }
}

class AdminMarketActionResult {
  const AdminMarketActionResult({
    required this.market,
    required this.engineActivated,
  });

  final AdminMarket market;
  final bool engineActivated;

  factory AdminMarketActionResult.fromJson(Map<String, dynamic> json) {
    final market = json['market'];
    if (market is! Map) {
      throw const ApiError('invalid market response');
    }
    return AdminMarketActionResult(
      market: AdminMarket.fromJson(Map<String, dynamic>.from(market)),
      engineActivated: json['engine_activated'] == true,
    );
  }
}

/// Admin system setup + base currency + platform packages + corp products + markets.
class AdminSystemProvider {
  AdminSystemProvider(this.api);

  final ApiClient api;

  Future<AdminSetupStatus> fetchSetup() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>('/v1/admin/system/setup');
      final setup = res.data?['setup'];
      if (setup is! Map) {
        throw const ApiError('invalid setup response');
      }
      return AdminSetupStatus.fromJson(Map<String, dynamic>.from(setup));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  Future<String> setBaseCurrency(String code) async {
    try {
      final res = await api.dio.put<Map<String, dynamic>>(
        '/v1/admin/system/base-currency',
        data: <String, dynamic>{'base_fiat_currency': code},
      );
      return res.data?['base_fiat_currency']?.toString() ?? code;
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  Future<List<AdminShopPackage>> listPackages() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>('/v1/admin/shop/packages');
      final raw = res.data?['packages'];
      if (raw is! List) {
        return const [];
      }
      return raw
          .whereType<Map>()
          .map((e) => AdminShopPackage.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  Future<AdminShopPackage> createPackage({
    required String code,
    required String name,
    required double coinAmount,
    required double fiatPrice,
    String? status,
  }) async {
    try {
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/admin/shop/packages',
        data: <String, dynamic>{
          'code': code,
          'name': name,
          'coin_amount': coinAmount,
          'fiat_price': fiatPrice,
          if (status != null) 'status': status,
        },
      );
      final pkg = res.data?['package'];
      if (pkg is! Map) {
        throw const ApiError('invalid package response');
      }
      return AdminShopPackage.fromJson(Map<String, dynamic>.from(pkg));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  Future<AdminShopPackage> updatePackage({
    required int id,
    required double fiatPrice,
    String? name,
    String? status,
  }) async {
    try {
      final res = await api.dio.put<Map<String, dynamic>>(
        '/v1/admin/shop/packages/$id',
        data: <String, dynamic>{
          'fiat_price': fiatPrice,
          if (name != null) 'name': name,
          if (status != null) 'status': status,
        },
      );
      final pkg = res.data?['package'];
      if (pkg is! Map) {
        throw const ApiError('invalid package response');
      }
      return AdminShopPackage.fromJson(Map<String, dynamic>.from(pkg));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  Future<AdminShopPackage> setPackageStatus({
    required int id,
    required String status,
  }) async {
    try {
      final res = await api.dio.patch<Map<String, dynamic>>(
        '/v1/admin/shop/packages/$id/status',
        data: <String, dynamic>{'status': status},
      );
      final pkg = res.data?['package'];
      if (pkg is! Map) {
        throw const ApiError('invalid package response');
      }
      return AdminShopPackage.fromJson(Map<String, dynamic>.from(pkg));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  Future<AdminSetupStatus> completeSetup() async {
    try {
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/admin/system/setup/complete',
      );
      final setup = res.data?['setup'];
      if (setup is! Map) {
        throw const ApiError('invalid setup response');
      }
      return AdminSetupStatus.fromJson(Map<String, dynamic>.from(setup));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  Future<List<AdminCorpProduct>> listCorpProducts({
    String? status,
    int? corporateUserId,
  }) async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>(
        '/v1/admin/shop/corp-products',
        queryParameters: <String, dynamic>{
          if (status != null && status.isNotEmpty) 'status': status,
          if (corporateUserId != null) 'corporate_user_id': corporateUserId,
        },
      );
      final raw = res.data?['products'];
      if (raw is! List) {
        return const [];
      }
      return raw
          .whereType<Map>()
          .map((e) => AdminCorpProduct.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  Future<AdminCorpProduct> setCorpProductStatus({
    required int id,
    required String status,
  }) async {
    try {
      final res = await api.dio.patch<Map<String, dynamic>>(
        '/v1/admin/shop/corp-products/$id/status',
        data: <String, dynamic>{'status': status},
      );
      final product = res.data?['product'];
      if (product is! Map) {
        throw const ApiError('invalid product response');
      }
      return AdminCorpProduct.fromJson(Map<String, dynamic>.from(product));
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  Future<List<AdminMarket>> listMarkets({String? status}) async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>(
        '/v1/admin/markets',
        queryParameters: <String, dynamic>{
          if (status != null && status.isNotEmpty) 'status': status,
        },
      );
      final raw = res.data?['markets'];
      if (raw is! List) {
        return const [];
      }
      return raw
          .whereType<Map>()
          .map((e) => AdminMarket.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  Future<AdminMarketActionResult> approveMarket(int id) async {
    try {
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/admin/markets/$id/approve',
      );
      return AdminMarketActionResult.fromJson(res.data ?? const {});
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  Future<AdminMarket> rejectMarket(int id) async {
    try {
      final res = await api.dio.post<Map<String, dynamic>>(
        '/v1/admin/markets/$id/reject',
      );
      final market = res.data?['market'];
      if (market is! Map) {
        throw const ApiError('invalid market response');
      }
      return AdminMarket.fromJson(Map<String, dynamic>.from(market));
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
