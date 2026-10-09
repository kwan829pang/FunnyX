import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/cart_controller.dart';
import '../../controllers/shop_catalog_controller.dart';
import '../../l10n/app_localizations.dart';

class CartPage extends StatelessWidget {
  const CartPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final cart = Get.find<CartController>();
    final fiat = Get.find<ShopCatalogController>().fiatCurrency;

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              IconButton(
                onPressed: () => Get.back(),
                icon: const Icon(Icons.arrow_back),
              ),
              Expanded(
                child: Text(
                  l10n.shopCart,
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Expanded(
            child: Obx(() {
              if (cart.lines.isEmpty) {
                return Center(child: Text(l10n.shopCartEmpty));
              }
              return SingleChildScrollView(
                child: DataTable(
                  columns: [
                    DataColumn(label: Text(l10n.shopColItem)),
                    DataColumn(label: Text(l10n.shopColUnitPrice)),
                    DataColumn(label: Text(l10n.shopColQty)),
                    DataColumn(label: Text(l10n.shopColLineTotal)),
                    DataColumn(label: Text(l10n.shopColActions)),
                  ],
                  rows: cart.lines.map((line) {
                    return DataRow(
                      cells: [
                        DataCell(
                          Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              Text(line.name),
                              Text(
                                line.code,
                                style: Theme.of(context).textTheme.bodySmall,
                              ),
                            ],
                          ),
                        ),
                        DataCell(
                          Text(
                            '${line.fiatPrice.toStringAsFixed(2)} ${fiat.value}',
                          ),
                        ),
                        DataCell(
                          Row(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              IconButton(
                                icon: const Icon(Icons.remove),
                                onPressed: () =>
                                    cart.setQty(line.key, line.qty - 1),
                              ),
                              Text('${line.qty}'),
                              IconButton(
                                icon: const Icon(Icons.add),
                                onPressed: () =>
                                    cart.setQty(line.key, line.qty + 1),
                              ),
                            ],
                          ),
                        ),
                        DataCell(
                          Text(
                            '${line.lineTotal.toStringAsFixed(2)} ${fiat.value}',
                          ),
                        ),
                        DataCell(
                          IconButton(
                            icon: const Icon(Icons.delete_outline),
                            onPressed: () => cart.remove(line.key),
                          ),
                        ),
                      ],
                    );
                  }).toList(),
                ),
              );
            }),
          ),
          const Divider(),
          Obx(
            () => Row(
              children: [
                Expanded(
                  child: Text(
                    l10n.shopSubtotal(
                      '${cart.subtotal.toStringAsFixed(2)} ${fiat.value}',
                    ),
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
                FilledButton(
                  onPressed: cart.lines.isEmpty
                      ? null
                      : () => Get.toNamed('/shop/checkout'),
                  child: Text(l10n.shopCheckout),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
