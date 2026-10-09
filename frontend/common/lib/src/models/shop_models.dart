/// E-shop catalog and order models (Client Center `/v1/shop/*`).

class ShopPackage {
  const ShopPackage({
    required this.id,
    required this.code,
    required this.name,
    required this.gameCoinId,
    required this.creditGameCoin,
    required this.coinAmount,
    required this.fiatPrice,
    required this.sellerType,
    required this.status,
  });

  final int id;
  final String code;
  final String name;
  final int gameCoinId;
  final String creditGameCoin;
  final double coinAmount;
  final double fiatPrice;
  final String sellerType;
  final String status;

  factory ShopPackage.fromJson(Map<String, dynamic> json) {
    return ShopPackage(
      id: _asInt(json['id']),
      code: json['code']?.toString() ?? '',
      name: json['name']?.toString() ?? '',
      gameCoinId: _asInt(json['game_coin_id']),
      creditGameCoin: json['credit_game_coin']?.toString() ?? '',
      coinAmount: _asDouble(json['coin_amount']),
      fiatPrice: _asDouble(json['fiat_price']),
      sellerType: json['seller_type']?.toString() ?? 'platform',
      status: json['status']?.toString() ?? '',
    );
  }
}

class CorpProduct {
  const CorpProduct({
    required this.id,
    required this.corporateUserId,
    this.gameId,
    required this.code,
    required this.name,
    required this.productType,
    this.creditGameCoinId,
    this.creditGameCoin,
    this.creditAmount,
    this.itemCode,
    required this.fiatPrice,
    required this.sellerType,
    required this.status,
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
  final String sellerType;
  final String status;

  factory CorpProduct.fromJson(Map<String, dynamic> json) {
    return CorpProduct(
      id: _asInt(json['id']),
      corporateUserId: _asInt(json['corporate_user_id']),
      gameId: _asIntOrNull(json['game_id']),
      code: json['code']?.toString() ?? '',
      name: json['name']?.toString() ?? '',
      productType: json['product_type']?.toString() ?? '',
      creditGameCoinId: _asIntOrNull(json['credit_game_coin_id']),
      creditGameCoin: json['credit_game_coin']?.toString(),
      creditAmount: json['credit_amount'] == null
          ? null
          : _asDouble(json['credit_amount']),
      itemCode: json['item_code']?.toString(),
      fiatPrice: _asDouble(json['fiat_price']),
      sellerType: json['seller_type']?.toString() ?? 'corp',
      status: json['status']?.toString() ?? '',
    );
  }
}

class ShopOrder {
  const ShopOrder({
    required this.id,
    required this.endUserId,
    required this.gameAccountId,
    required this.sellerType,
    this.packageId,
    this.corpProductId,
    this.packageCode,
    this.productCode,
    this.creditGameCoin,
    this.creditAmount,
    this.itemCode,
    required this.fiatCurrency,
    required this.fiatPrice,
    this.partnerOrderNo,
    this.checkoutUrl,
    required this.status,
    required this.expiresAt,
    required this.paidAt,
    required this.createdAt,
  });

  final int id;
  final int endUserId;
  final int gameAccountId;
  final String sellerType;
  final int? packageId;
  final int? corpProductId;
  final String? packageCode;
  final String? productCode;
  final String? creditGameCoin;
  final double? creditAmount;
  final String? itemCode;
  final String fiatCurrency;
  final double fiatPrice;
  final String? partnerOrderNo;
  final String? checkoutUrl;
  final String status;
  final int expiresAt;
  final int paidAt;
  final int createdAt;

  String get displayCode =>
      packageCode ?? productCode ?? itemCode ?? '#$id';

  bool get isPending => status == 'pending';

  factory ShopOrder.fromJson(Map<String, dynamic> json) {
    return ShopOrder(
      id: _asInt(json['id']),
      endUserId: _asInt(json['end_user_id']),
      gameAccountId: _asInt(json['game_account_id']),
      sellerType: json['seller_type']?.toString() ?? '',
      packageId: _asIntOrNull(json['package_id']),
      corpProductId: _asIntOrNull(json['corp_product_id']),
      packageCode: json['package_code']?.toString(),
      productCode: json['product_code']?.toString(),
      creditGameCoin: json['credit_game_coin']?.toString(),
      creditAmount: json['credit_amount'] == null
          ? null
          : _asDouble(json['credit_amount']),
      itemCode: json['item_code']?.toString(),
      fiatCurrency: json['fiat_currency']?.toString() ?? '',
      fiatPrice: _asDouble(json['fiat_price']),
      partnerOrderNo: json['partner_order_no']?.toString(),
      checkoutUrl: json['checkout_url']?.toString(),
      status: json['status']?.toString() ?? '',
      expiresAt: _asInt(json['expires_at']),
      paidAt: _asInt(json['paid_at']),
      createdAt: _asInt(json['created_at']),
    );
  }
}

class GameAccountBinding {
  const GameAccountBinding({
    required this.id,
    required this.endUserId,
    required this.gameId,
    required this.gameCode,
    required this.gameAccountId,
    required this.partnerId,
    this.partnerUserId,
    required this.bindSource,
    required this.status,
  });

  final int id;
  final int endUserId;
  final int gameId;
  final String gameCode;
  final String gameAccountId;
  final String partnerId;
  final String? partnerUserId;
  final String bindSource;
  final String status;

  String get label => '$gameCode · $gameAccountId';

  factory GameAccountBinding.fromJson(Map<String, dynamic> json) {
    return GameAccountBinding(
      id: _asInt(json['id']),
      endUserId: _asInt(json['end_user_id']),
      gameId: _asInt(json['game_id']),
      gameCode: json['game_code']?.toString() ?? '',
      gameAccountId: json['game_account_id']?.toString() ?? '',
      partnerId: json['partner_id']?.toString() ?? '',
      partnerUserId: json['partner_user_id']?.toString(),
      bindSource: json['bind_source']?.toString() ?? '',
      status: json['status']?.toString() ?? '',
    );
  }
}

/// Local cart line (client-side; one API order per line at checkout).
class CartLine {
  CartLine({
    required this.sellerType,
    required this.catalogId,
    required this.code,
    required this.name,
    required this.fiatPrice,
    this.qty = 1,
  });

  final String sellerType;
  final int catalogId;
  final String code;
  final String name;
  final double fiatPrice;
  int qty;

  String get key => '$sellerType:$catalogId';

  double get lineTotal => fiatPrice * qty;
}

int _asInt(dynamic v) {
  if (v is int) return v;
  if (v is num) return v.toInt();
  return int.tryParse(v?.toString() ?? '') ?? 0;
}

int? _asIntOrNull(dynamic v) {
  if (v == null) return null;
  return _asInt(v);
}

double _asDouble(dynamic v) {
  if (v is double) return v;
  if (v is num) return v.toDouble();
  return double.tryParse(v?.toString() ?? '') ?? 0;
}
