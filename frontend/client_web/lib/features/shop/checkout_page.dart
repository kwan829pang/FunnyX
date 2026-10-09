import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import '../../controllers/cart_controller.dart';
import '../../controllers/shop_catalog_controller.dart';
import '../../controllers/shop_checkout_controller.dart';
import '../../l10n/app_localizations.dart';

class CheckoutPage extends StatefulWidget {
  const CheckoutPage({super.key});

  @override
  State<CheckoutPage> createState() => _CheckoutPageState();
}

class _CheckoutPageState extends State<CheckoutPage> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      Get.find<ShopCheckoutController>().loadBindings();
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final cart = Get.find<CartController>();
    final checkout = Get.find<ShopCheckoutController>();
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
                  l10n.shopCheckout,
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Text(l10n.shopSelectGameAccount),
          const SizedBox(height: 8),
          Obx(() {
            if (checkout.loading.value) {
              return const LinearProgressIndicator();
            }
            if (checkout.bindings.isEmpty) {
              return Text(
                l10n.shopNoGameAccount,
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              );
            }
            return DropdownMenu<int>(
              initialSelection: checkout.selectedBindingId.value,
              label: Text(l10n.shopGameAccount),
              expandedInsets: EdgeInsets.zero,
              dropdownMenuEntries: checkout.bindings
                  .map(
                    (b) => DropdownMenuEntry(value: b.id, label: b.label),
                  )
                  .toList(),
              onSelected: (v) => checkout.selectedBindingId.value = v,
            );
          }),
          const SizedBox(height: 16),
          Text(l10n.shopOrderSummary),
          const SizedBox(height: 8),
          Expanded(
            child: Obx(() {
              if (cart.lines.isEmpty) {
                return Center(child: Text(l10n.shopCartEmpty));
              }
              return SingleChildScrollView(
                child: DataTable(
                  columns: [
                    DataColumn(label: Text(l10n.shopColItem)),
                    DataColumn(label: Text(l10n.shopColQty)),
                    DataColumn(label: Text(l10n.shopColLineTotal)),
                  ],
                  rows: cart.lines
                      .map(
                        (line) => DataRow(
                          cells: [
                            DataCell(Text('${line.name} (${line.code})')),
                            DataCell(Text('${line.qty}')),
                            DataCell(
                              Text(
                                '${line.lineTotal.toStringAsFixed(2)} ${fiat.value}',
                              ),
                            ),
                          ],
                        ),
                      )
                      .toList(),
                ),
              );
            }),
          ),
          Obx(
            () => checkout.error.value == null
                ? const SizedBox.shrink()
                : Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: Text(
                      checkout.error.value!,
                      style: TextStyle(
                        color: Theme.of(context).colorScheme.error,
                      ),
                    ),
                  ),
          ),
          Obx(
            () => FilledButton(
              onPressed: checkout.submitting.value ||
                      cart.lines.isEmpty ||
                      checkout.bindings.isEmpty
                  ? null
                  : () async {
                      try {
                        final orders = await checkout.placeOrders(
                          returnUrl:
                              '${Uri.base.origin}/#/shop/payment',
                        );
                        final ids = orders.map((o) => o.id).toList();
                        Get.offNamed(
                          '/shop/payment',
                          arguments: {'orderIds': ids},
                        );
                      } catch (e) {
                        if (!context.mounted) return;
                        toastification.show(
                          context: context,
                          type: ToastificationType.error,
                          title: Text(e.toString()),
                          autoCloseDuration: const Duration(seconds: 4),
                        );
                      }
                    },
              child: checkout.submitting.value
                  ? const SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : Text(l10n.shopPlaceOrder),
            ),
          ),
        ],
      ),
    );
  }
}
