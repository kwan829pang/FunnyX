import 'package:dio/dio.dart';

import '../models/api_error.dart';
import 'api_client.dart';

class ClientPartnerLink {
  const ClientPartnerLink({
    required this.partnerId,
    required this.partnerUserId,
    this.gameId,
    this.gameAccountId,
  });

  final String partnerId;
  final String partnerUserId;
  final int? gameId;
  final String? gameAccountId;

  factory ClientPartnerLink.fromJson(Map<String, dynamic> json) {
    return ClientPartnerLink(
      partnerId: json['partner_id']?.toString() ?? '',
      partnerUserId: json['partner_user_id']?.toString() ?? '',
      gameId: (json['game_id'] as num?)?.toInt(),
      gameAccountId: json['game_account_id']?.toString(),
    );
  }
}

class ClientProfile {
  const ClientProfile({
    required this.endUserId,
    required this.username,
    this.email,
    required this.status,
    required this.partnerLinks,
  });

  final int endUserId;
  final String username;
  final String? email;
  final String status;
  final List<ClientPartnerLink> partnerLinks;

  factory ClientProfile.fromJson(Map<String, dynamic> json) {
    final linksRaw = json['partner_links'];
    return ClientProfile(
      endUserId: (json['end_user_id'] as num?)?.toInt() ?? 0,
      username: json['username']?.toString() ?? '',
      email: json['email']?.toString(),
      status: json['status']?.toString() ?? '',
      partnerLinks: linksRaw is List
          ? linksRaw
              .whereType<Map>()
              .map((e) => ClientPartnerLink.fromJson(Map<String, dynamic>.from(e)))
              .toList()
          : const [],
    );
  }
}

class ClientWalletBalance {
  const ClientWalletBalance({
    required this.gameAccountRowId,
    required this.gameAccountId,
    required this.gameId,
    required this.gameCode,
    required this.gameCoinId,
    required this.gameCoin,
    required this.available,
    required this.locked,
  });

  final int gameAccountRowId;
  final String gameAccountId;
  final int gameId;
  final String gameCode;
  final int gameCoinId;
  final String gameCoin;
  final double available;
  final double locked;

  factory ClientWalletBalance.fromJson(Map<String, dynamic> json) {
    return ClientWalletBalance(
      gameAccountRowId: (json['game_account_row_id'] as num?)?.toInt() ?? 0,
      gameAccountId: json['game_account_id']?.toString() ?? '',
      gameId: (json['game_id'] as num?)?.toInt() ?? 0,
      gameCode: json['game_code']?.toString() ?? '',
      gameCoinId: (json['game_coin_id'] as num?)?.toInt() ?? 0,
      gameCoin: json['game_coin']?.toString() ?? '',
      available: (json['available'] as num?)?.toDouble() ?? 0,
      locked: (json['locked'] as num?)?.toDouble() ?? 0,
    );
  }
}

class ClientMoneyTxn {
  const ClientMoneyTxn({
    required this.id,
    required this.endUserId,
    this.gameAccountId,
    required this.transactionType,
    required this.amount,
    required this.gameCoinId,
    this.gameCoin,
    required this.status,
    required this.createdAt,
    required this.updatedAt,
  });

  final int id;
  final int endUserId;
  final int? gameAccountId;
  final String transactionType;
  final double amount;
  final int gameCoinId;
  final String? gameCoin;
  final String status;
  final int createdAt;
  final int updatedAt;

  factory ClientMoneyTxn.fromJson(Map<String, dynamic> json) {
    return ClientMoneyTxn(
      id: (json['id'] as num?)?.toInt() ?? 0,
      endUserId: (json['end_user_id'] as num?)?.toInt() ?? 0,
      gameAccountId: (json['game_account_id'] as num?)?.toInt(),
      transactionType: json['transaction_type']?.toString() ?? '',
      amount: (json['amount'] as num?)?.toDouble() ?? 0,
      gameCoinId: (json['game_coin_id'] as num?)?.toInt() ?? 0,
      gameCoin: json['game_coin']?.toString(),
      status: json['status']?.toString() ?? '',
      createdAt: (json['created_at'] as num?)?.toInt() ?? 0,
      updatedAt: (json['updated_at'] as num?)?.toInt() ?? 0,
    );
  }
}

/// Client Center Session wallet / profile / money APIs.
abstract class WalletDataProvider {
  Future<ClientProfile> getProfile();

  Future<List<ClientWalletBalance>> listWallets();

  Future<List<ClientMoneyTxn>> listTransactions({int limit = 50});

  Future<ClientMoneyTxn> deposit({
    required int gameAccountId,
    required String gameCoin,
    required double amount,
  });

  Future<ClientMoneyTxn> withdraw({
    required int gameAccountId,
    required String gameCoin,
    required double amount,
  });
}

class ClientCenterWalletProvider implements WalletDataProvider {
  ClientCenterWalletProvider(this.api);

  final ApiClient api;

  @override
  Future<ClientProfile> getProfile() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>('/v1/client/profile');
      final data = res.data;
      if (data == null) {
        throw const ApiError('invalid profile response');
      }
      return ClientProfile.fromJson(data);
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<List<ClientWalletBalance>> listWallets() async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>('/v1/client/wallet');
      final raw = res.data?['wallets'];
      if (raw is! List) return const [];
      return raw
          .whereType<Map>()
          .map((e) => ClientWalletBalance.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<List<ClientMoneyTxn>> listTransactions({int limit = 50}) async {
    try {
      final res = await api.dio.get<Map<String, dynamic>>(
        '/v1/client/transactions',
        queryParameters: <String, dynamic>{'limit': limit},
      );
      final raw = res.data?['transactions'];
      if (raw is! List) return const [];
      return raw
          .whereType<Map>()
          .map((e) => ClientMoneyTxn.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    } on DioException catch (e) {
      throw _mapDio(e);
    }
  }

  @override
  Future<ClientMoneyTxn> deposit({
    required int gameAccountId,
    required String gameCoin,
    required double amount,
  }) async {
    return _submitMoney(
      path: '/v1/client/deposit',
      gameAccountId: gameAccountId,
      gameCoin: gameCoin,
      amount: amount,
    );
  }

  @override
  Future<ClientMoneyTxn> withdraw({
    required int gameAccountId,
    required String gameCoin,
    required double amount,
  }) async {
    return _submitMoney(
      path: '/v1/client/withdraw',
      gameAccountId: gameAccountId,
      gameCoin: gameCoin,
      amount: amount,
    );
  }

  Future<ClientMoneyTxn> _submitMoney({
    required String path,
    required int gameAccountId,
    required String gameCoin,
    required double amount,
  }) async {
    try {
      final res = await api.dio.post<Map<String, dynamic>>(
        path,
        data: <String, dynamic>{
          'game_account_id': gameAccountId,
          'game_coin': gameCoin,
          'amount': amount,
        },
      );
      final txn = res.data?['txn'];
      if (txn is! Map) {
        throw const ApiError('invalid money response');
      }
      return ClientMoneyTxn.fromJson(Map<String, dynamic>.from(txn));
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
