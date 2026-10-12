import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

class MarketplaceController extends GetxController {
  MarketplaceController(this._api, this._shop);

  final MarketplaceDataProvider _api;
  final ShopDataProvider _shop;

  final loading = false.obs;
  final error = RxnString();
  final eligibility = Rxn<MarketplaceEligibility>();
  final deals = <MarketplaceDeal>[].obs;
  final gameAccounts = <GameAccountBinding>[].obs;
  final items = <MarketplaceItem>[].obs;
  final selectedDeal = Rxn<MarketplaceDeal>();

  Future<void> loadList() async {
    loading.value = true;
    error.value = null;
    try {
      final results = await Future.wait([
        _api.eligibility(),
        _api.listDeals(),
        _shop.listGameAccounts(),
      ]);
      eligibility.value = results[0] as MarketplaceEligibility;
      deals.assignAll(results[1] as List<MarketplaceDeal>);
      gameAccounts.assignAll(results[2] as List<GameAccountBinding>);
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
    } finally {
      loading.value = false;
    }
  }

  Future<void> loadDeal(int id) async {
    loading.value = true;
    error.value = null;
    try {
      selectedDeal.value = await _api.getDeal(id);
      if (gameAccounts.isEmpty) {
        gameAccounts.assignAll(await _shop.listGameAccounts());
      }
      if (eligibility.value == null) {
        eligibility.value = await _api.eligibility();
      }
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
    } finally {
      loading.value = false;
    }
  }

  Future<void> loadItems(int gameAccountId) async {
    try {
      items.assignAll(await _api.listItems(gameAccountId: gameAccountId));
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
    }
  }

  Future<MarketplaceDeal?> place({
    required int posterGameAccountId,
    required String offerAssetType,
    int? offerGameCoinId,
    String? offerItemRefId,
    required double offerAmount,
    required String wantAssetType,
    required int wantGameCoinId,
    required double wantAmount,
  }) async {
    loading.value = true;
    error.value = null;
    try {
      final deal = await _api.placeDeal(
        posterGameAccountId: posterGameAccountId,
        offerAssetType: offerAssetType,
        offerGameCoinId: offerGameCoinId,
        offerItemRefId: offerItemRefId,
        offerAmount: offerAmount,
        wantAssetType: wantAssetType,
        wantGameCoinId: wantGameCoinId,
        wantAmount: wantAmount,
      );
      await loadList();
      return deal;
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
      return null;
    } finally {
      loading.value = false;
    }
  }

  Future<bool> take({
    required int dealId,
    required int requesterGameAccountId,
  }) async {
    loading.value = true;
    error.value = null;
    try {
      selectedDeal.value = await _api.takeDeal(
        dealId: dealId,
        requesterGameAccountId: requesterGameAccountId,
      );
      await loadList();
      return true;
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
      return false;
    } finally {
      loading.value = false;
    }
  }

  Future<bool> cancel(int dealId) async {
    loading.value = true;
    error.value = null;
    try {
      selectedDeal.value = await _api.cancelDeal(dealId);
      await loadList();
      return true;
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
      return false;
    } finally {
      loading.value = false;
    }
  }
}
