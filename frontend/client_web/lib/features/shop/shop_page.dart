import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import '../../controllers/cart_controller.dart';
import '../../controllers/shop_catalog_controller.dart';
import '../../l10n/app_localizations.dart';
import 'widgets/product_card.dart';

class ShopPage extends StatefulWidget {
  const ShopPage({super.key});

  @override
  State<ShopPage> createState() => _ShopPageState();
}

class _ShopPageState extends State<ShopPage> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      Get.find<ShopCatalogController>().load();
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final catalog = Get.find<ShopCatalogController>();
    final cart = Get.find<CartController>();

    return ResponsiveScope(
      builder: (context, size) {
        final cols = switch (size) {
          AppLayoutSize.mobile => 2,
          AppLayoutSize.tablet => 3,
          AppLayoutSize.desktop => 4,
        };
        return Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                children: [
                  Expanded(
                    child: Text(
                      l10n.shop,
                      style: Theme.of(context).textTheme.headlineSmall,
                    ),
                  ),
                  TextButton(
                    onPressed: () => Get.toNamed('/shop/orders'),
                    child: Text(l10n.shopOrders),
                  ),
                  const SizedBox(width: 8),
                  Obx(
                    () => Badge(
                      isLabelVisible: cart.itemCount > 0,
                      label: Text('${cart.itemCount}'),
                      child: IconButton(
                        tooltip: l10n.shopCart,
                        onPressed: () => Get.toNamed('/shop/cart'),
                        icon: const Icon(Icons.shopping_cart_outlined),
                      ),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 8),
              Obx(
                () => Text(
                  l10n.shopBaseCurrency(catalog.fiatCurrency.value),
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
              ),
              const SizedBox(height: 12),
              Expanded(
                child: Obx(() {
                  if (catalog.loading.value &&
                      catalog.packages.isEmpty &&
                      catalog.corpProducts.isEmpty) {
                    return const Center(child: CircularProgressIndicator());
                  }
                  if (catalog.error.value != null &&
                      catalog.packages.isEmpty &&
                      catalog.corpProducts.isEmpty) {
                    return Center(
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Text(catalog.error.value!),
                          const SizedBox(height: 12),
                          FilledButton(
                            onPressed: catalog.load,
                            child: Text(l10n.shopRetry),
                          ),
                        ],
                      ),
                    );
                  }
                  final items = <_CatalogItem>[
                    ...catalog.packages.map(
                      (p) => _CatalogItem.platform(p, catalog.fiatCurrency.value),
                    ),
                    ...catalog.corpProducts.map(
                      (p) => _CatalogItem.corp(p, catalog.fiatCurrency.value),
                    ),
                  ];
                  if (items.isEmpty) {
                    return Center(child: Text(l10n.shopEmptyCatalog));
                  }
                  return RefreshIndicator(
                    onRefresh: catalog.load,
                    child: GridView.builder(
                      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                        crossAxisCount: cols,
                        mainAxisSpacing: 12,
                        crossAxisSpacing: 12,
                        childAspectRatio: 0.95,
                      ),
                      itemCount: items.length,
                      itemBuilder: (context, i) {
                        final item = items[i];
                        return ProductCard(
                          title: item.name,
                          code: item.code,
                          sellerLabel: item.sellerLabel(l10n),
                          priceLabel:
                              '${item.fiatPrice.toStringAsFixed(2)} ${item.fiat}',
                          onAdd: () {
                            item.addToCart(cart);
                            toastification.show(
                              context: context,
                              type: ToastificationType.success,
                              title: Text(l10n.shopAddedToCart),
                              autoCloseDuration: const Duration(seconds: 2),
                            );
                          },
                        );
                      },
                    ),
                  );
                }),
              ),
            ],
          ),
        );
      },
    );
  }
}

class _CatalogItem {
  _CatalogItem._({
    required this.name,
    required this.code,
    required this.fiatPrice,
    required this.fiat,
    required this.isPlatform,
    required this.addToCart,
  });

  factory _CatalogItem.platform(ShopPackage p, String fiat) => _CatalogItem._(
        name: p.name,
        code: p.code,
        fiatPrice: p.fiatPrice,
        fiat: fiat,
        isPlatform: true,
        addToCart: (c) => c.addPackage(p),
      );

  factory _CatalogItem.corp(CorpProduct p, String fiat) => _CatalogItem._(
        name: p.name,
        code: p.code,
        fiatPrice: p.fiatPrice,
        fiat: fiat,
        isPlatform: false,
        addToCart: (c) => c.addCorpProduct(p),
      );

  final String name;
  final String code;
  final double fiatPrice;
  final String fiat;
  final bool isPlatform;
  final void Function(CartController) addToCart;

  String sellerLabel(AppLocalizations l10n) =>
      isPlatform ? l10n.shopSellerPlatform : l10n.shopSellerCorp;
}
