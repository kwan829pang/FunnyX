import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/shop_orders_controller.dart';
import '../../l10n/app_localizations.dart';

class OrdersPage extends StatefulWidget {
  const OrdersPage({super.key});

  @override
  State<OrdersPage> createState() => _OrdersPageState();
}

class _OrdersPageState extends State<OrdersPage> {
  late final ShopOrdersController orders;

  @override
  void initState() {
    super.initState();
    orders = Get.find<ShopOrdersController>();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      orders.loadOrders();
    });
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
                onPressed: () => Get.back(),
                icon: const Icon(Icons.arrow_back),
              ),
              Expanded(
                child: Text(
                  l10n.shopOrders,
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
              ),
              IconButton(
                onPressed: () => orders.loadOrders(),
                icon: const Icon(Icons.refresh),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Expanded(
            child: Obx(() {
              if (orders.loading.value && orders.orders.isEmpty) {
                return const Center(child: CircularProgressIndicator());
              }
              if (orders.error.value != null && orders.orders.isEmpty) {
                return Center(child: Text(orders.error.value!));
              }
              if (orders.orders.isEmpty) {
                return Center(child: Text(l10n.shopNoOrders));
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
                    ],
                    rows: orders.orders
                        .map(
                          (o) => DataRow(
                            onSelectChanged: (_) =>
                                Get.toNamed('/shop/orders/${o.id}'),
                            cells: [
                              DataCell(Text('#${o.id}')),
                              DataCell(Text(o.displayCode)),
                              DataCell(Text(o.sellerType)),
                              DataCell(
                                Text(
                                  '${o.fiatPrice.toStringAsFixed(2)} ${o.fiatCurrency}',
                                ),
                              ),
                              DataCell(Text(o.status)),
                              DataCell(Text(_fmtExpires(o.expiresAt))),
                            ],
                          ),
                        )
                        .toList(),
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
