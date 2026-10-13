/// C7 Marketplace deals (Client Center `/v1/marketplace/*`).
library;

class MarketplaceEligibility {
  const MarketplaceEligibility({
    required this.canPlace,
    required this.verifiedPlatformToken,
    required this.playingListedGame,
    required this.transferApiOpen,
    required this.listingFeePlt,
    required this.reasons,
  });

  final bool canPlace;
  final bool verifiedPlatformToken;
  final bool playingListedGame;
  final bool transferApiOpen;
  final double listingFeePlt;
  final List<String> reasons;

  factory MarketplaceEligibility.fromJson(Map<String, dynamic> json) {
    final reasonsRaw = json['reasons'];
    return MarketplaceEligibility(
      canPlace: json['can_place'] == true,
      verifiedPlatformToken: json['verified_platform_token'] == true,
      playingListedGame: json['playing_listed_game'] == true,
      transferApiOpen: json['transfer_api_open'] == true,
      listingFeePlt: _asDouble(json['listing_fee_plt'] ?? 1),
      reasons: reasonsRaw is List
          ? reasonsRaw.map((e) => e.toString()).toList()
          : const [],
    );
  }
}

class MarketplaceItem {
  const MarketplaceItem({
    required this.itemRefId,
    required this.name,
    required this.qty,
    required this.gameAccountId,
  });

  final String itemRefId;
  final String name;
  final int qty;
  final String gameAccountId;

  factory MarketplaceItem.fromJson(Map<String, dynamic> json) {
    return MarketplaceItem(
      itemRefId: json['item_ref_id']?.toString() ?? '',
      name: json['name']?.toString() ?? '',
      qty: _asInt(json['qty']),
      gameAccountId: json['game_account_id']?.toString() ?? '',
    );
  }
}

class MarketplaceDeal {
  const MarketplaceDeal({
    required this.id,
    required this.posterEndUserId,
    required this.posterGameAccountId,
    required this.gameId,
    required this.gameCode,
    required this.corporateUserId,
    required this.offerAssetType,
    this.offerGameCoinId,
    required this.offerGameCoin,
    this.offerItemRefId,
    required this.offerAmount,
    required this.wantAssetType,
    required this.wantGameCoinId,
    required this.wantGameCoin,
    required this.wantAmount,
    required this.status,
    required this.createdAt,
    required this.listingFeePlt,
  });

  final int id;
  final int posterEndUserId;
  final int posterGameAccountId;
  final int gameId;
  final String gameCode;
  final int corporateUserId;
  final String offerAssetType;
  final int? offerGameCoinId;
  final String offerGameCoin;
  final String? offerItemRefId;
  final double offerAmount;
  final String wantAssetType;
  final int wantGameCoinId;
  final String wantGameCoin;
  final double wantAmount;
  final String status;
  final int createdAt;
  final double listingFeePlt;

  factory MarketplaceDeal.fromJson(Map<String, dynamic> json) {
    return MarketplaceDeal(
      id: _asInt(json['id']),
      posterEndUserId: _asInt(json['poster_end_user_id']),
      posterGameAccountId: _asInt(json['poster_game_account_id']),
      gameId: _asInt(json['game_id']),
      gameCode: json['game_code']?.toString() ?? '',
      corporateUserId: _asInt(json['corporate_user_id']),
      offerAssetType: json['offer_asset_type']?.toString() ?? '',
      offerGameCoinId: json['offer_game_coin_id'] == null
          ? null
          : _asInt(json['offer_game_coin_id']),
      offerGameCoin: json['offer_game_coin']?.toString() ?? '',
      offerItemRefId: json['offer_item_ref_id']?.toString(),
      offerAmount: _asDouble(json['offer_amount']),
      wantAssetType: json['want_asset_type']?.toString() ?? '',
      wantGameCoinId: _asInt(json['want_game_coin_id']),
      wantGameCoin: json['want_game_coin']?.toString() ?? '',
      wantAmount: _asDouble(json['want_amount']),
      status: json['status']?.toString() ?? '',
      createdAt: _asInt(json['created_at']),
      listingFeePlt: _asDouble(json['listing_fee_plt'] ?? 1),
    );
  }

  String get offerLabel {
    if (offerAssetType == 'game_item') {
      return offerItemRefId ?? 'item';
    }
    return '$offerAmount $offerGameCoin';
  }

  String get wantLabel => '$wantAmount $wantGameCoin';
}

int _asInt(dynamic v) {
  if (v is int) return v;
  if (v is num) return v.toInt();
  return int.tryParse(v?.toString() ?? '') ?? 0;
}

double _asDouble(dynamic v) {
  if (v is double) return v;
  if (v is num) return v.toDouble();
  return double.tryParse(v?.toString() ?? '') ?? 0;
}
