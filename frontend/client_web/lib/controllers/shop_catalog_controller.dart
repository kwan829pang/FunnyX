import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

class ShopCatalogController extends GetxController {
  ShopCatalogController(this.shop);

  final ShopDataProvider shop;

  final loading = false.obs;
  final error = RxnString();
  final fiatCurrency = 'HKD'.obs;
  final packages = <ShopPackage>[].obs;
  final corpProducts = <CorpProduct>[].obs;

  Future<void> load() async {
    loading.value = true;
    error.value = null;
    try {
      final results = await Future.wait([
        shop.baseCurrency(),
        shop.listPackages(),
        shop.listCorpProducts(),
      ]);
      fiatCurrency.value = results[0] as String;
      packages.assignAll(results[1] as List<ShopPackage>);
      corpProducts.assignAll(results[2] as List<CorpProduct>);
    } catch (e) {
      error.value = e is ApiError ? e.message : e.toString();
    } finally {
      loading.value = false;
    }
  }
}
