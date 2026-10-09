import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../controllers/shop_checkout_controller.dart';
import '../../controllers/shop_orders_controller.dart';
import '../../l10n/app_localizations.dart';

class PaymentPage extends StatefulWidget {
  const PaymentPage({super.key});

  @override
  State<PaymentPage> createState() => _PaymentPageState();
}

class _PaymentPageState extends State<PaymentPage> {
  late final ShopOrdersController orders;

  @override
  void initState() {
    super.initState();
    orders = Get.find<ShopOrdersController>();
    WidgetsBinding.instance.addPostFrameCallback((_) => _load());
  }

  Future<void> _load() async {
    final args = Get.arguments;
    final ids = <int>[];
    if (args is Map && args['orderIds'] is List) {
      for (final v in args['orderIds'] as List) {
        if (v is int) ids.add(v);
      }
    }
    final checkout = Get.find<ShopCheckoutController>();
    if (ids.isEmpty && checkout.lastCreatedOrders.isNotEmpty) {
      ids.addAll(checkout.lastCreatedOrders.map((o) => o.id));
    }
    await orders.loadOrders(pendingOnly: false);
    if (ids.isNotEmpty) {
      final filtered =
          orders.orders.where((o) => ids.contains(o.id)).toList();
      if (filtered.isNotEmpty) {
        orders.orders.assignAll(filtered);
      }
    } else {
      await orders.loadOrders(pendingOnly: true);
    }
    orders.startPendingPoll();
  }

  @override
  void dispose() {
    orders.stopPoll();
    super.dispose();
  }

  String _fmtExpires(int ms) {
    if (ms <= 0) return '—';
    final dt = DateTime.fromMillisecondsSinceEpoch(ms, isUtc: true).toLocal();
    return dt.toString().split('.').first;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              IconButton(
                onPressed: () => Get.offNamed('/shop'),
                icon: const Icon(Icons.arrow_back),
              ),
              Expanded(
                child: Text(
                  l10n.shopPayment,
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
              ),
              TextButton(
                onPressed: () => Get.toNamed('/shop/orders'),
                child: Text(l10n.shopOrders),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            l10n.shopPaymentHint,
            style: Theme.of(context).textTheme.bodyMedium,
          ),
          const SizedBox(height: 12),
          Expanded(
            child: Obx(() {
              if (orders.loading.value && orders.orders.isEmpty) {
                return const Center(child: CircularProgressIndicator());
              }
              if (orders.orders.isEmpty) {
                return Center(child: Text(l10n.shopNoPendingOrders));
              }
              return SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: SingleChildScrollView(
                  child: DataTable(
                    columns: [
                      DataColumn(label: Text(l10n.shopColOrderId)),
                      DataColumn(label: Text(l10n.shopColItem)),
                      DataColumn(label: Text(l10n.shopColSeller)),
                      DataColumn(label: Text(l10n.shopColFiat)),
                      DataColumn(label: Text(l10n.shopColStatus)),
                      DataColumn(label: Text(l10n.shopColExpires)),
                      DataColumn(label: Text(l10n.shopColActions)),
                    ],
                    rows: orders.orders.map((o) {
                      return DataRow(
                        cells: [
                          DataCell(
                            InkWell(
                              onTap: () =>
                                  Get.toNamed('/shop/orders/${o.id}'),
                              child: Text('#${o.id}'),
                            ),
                          ),
                          DataCell(Text(o.displayCode)),
                          DataCell(Text(o.sellerType)),
                          DataCell(
                            Text(
                              '${o.fiatPrice.toStringAsFixed(2)} ${o.fiatCurrency}',
                            ),
                          ),
                          DataCell(Text(o.status)),
                          DataCell(Text(_fmtExpires(o.expiresAt))),
                          DataCell(
                            Row(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                if (o.checkoutUrl != null &&
                                    o.checkoutUrl!.isNotEmpty)
                                  TextButton(
                                    onPressed: () async {
                                      final uri = Uri.tryParse(o.checkoutUrl!);
                                      if (uri == null) return;
                                      await launchUrl(
                                        uri,
                                        mode: LaunchMode.externalApplication,
                                      );
                                    },
                                    child: Text(l10n.shopPay),
                                  ),
                                if (o.isPending)
                                  TextButton(
                                    onPressed: () async {
                                      try {
                                        await orders.cancel(o.id);
                                        if (!context.mounted) return;
                                        toastification.show(
                                          context: context,
                                          type: ToastificationType.info,
                                          title: Text(l10n.shopCancelled),
                                          autoCloseDuration:
                                              const Duration(seconds: 2),
                                        );
                                      } catch (e) {
                                        if (!context.mounted) return;
                                        toastification.show(
                                          context: context,
                                          type: ToastificationType.error,
                                          title: Text(e.toString()),
                                          autoCloseDuration:
                                              const Duration(seconds: 3),
                                        );
                                      }
                                    },
                                    child: Text(l10n.shopCancel),
                                  ),
                              ],
                            ),
                          ),
                        ],
                      );
                    }).toList(),
                  ),
                ),
              );
            }),
          ),
        ],
      ),
    );
  }
}
