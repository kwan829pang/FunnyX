import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../controllers/shop_orders_controller.dart';
import '../../l10n/app_localizations.dart';

class OrderDetailPage extends StatefulWidget {
  const OrderDetailPage({super.key, required this.orderId});

  final int orderId;

  @override
  State<OrderDetailPage> createState() => _OrderDetailPageState();
}

class _OrderDetailPageState extends State<OrderDetailPage> {
  late final ShopOrdersController orders;

  @override
  void initState() {
    super.initState();
    orders = Get.find<ShopOrdersController>();
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      await orders.loadDetail(widget.orderId);
      orders.startDetailPoll(widget.orderId);
    });
  }

  @override
  void dispose() {
    orders.stopPoll();
    super.dispose();
  }

  String _fmtMs(int ms) {
    if (ms <= 0) return '—';
    return DateTime.fromMillisecondsSinceEpoch(ms, isUtc: true)
        .toLocal()
        .toString()
        .split('.')
        .first;
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
                  l10n.shopOrderDetail(widget.orderId),
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Expanded(
            child: Obx(() {
              if (orders.loading.value && orders.detail.value == null) {
                return const Center(child: CircularProgressIndicator());
              }
              final o = orders.detail.value;
              if (o == null) {
                return Center(
                  child: Text(orders.error.value ?? l10n.shopOrderNotFound),
                );
              }
              return ListView(
                children: [
                  _row(l10n.shopColStatus, o.status),
                  _row(l10n.shopColItem, o.displayCode),
                  _row(l10n.shopColSeller, o.sellerType),
                  _row(
                    l10n.shopColFiat,
                    '${o.fiatPrice.toStringAsFixed(2)} ${o.fiatCurrency}',
                  ),
                  if (o.creditGameCoin != null)
                    _row(
                      l10n.shopCredit,
                      '${o.creditAmount ?? 0} ${o.creditGameCoin}',
                    ),
                  _row(l10n.shopColExpires, _fmtMs(o.expiresAt)),
                  _row(l10n.shopCreatedAt, _fmtMs(o.createdAt)),
                  if (o.partnerOrderNo != null)
                    _row(l10n.shopPartnerOrder, o.partnerOrderNo!),
                  const SizedBox(height: 16),
                  if (o.checkoutUrl != null &&
                      o.checkoutUrl!.isNotEmpty &&
                      o.isPending)
                    FilledButton(
                      onPressed: () async {
                        final uri = Uri.tryParse(o.checkoutUrl!);
                        if (uri != null) {
                          await launchUrl(
                            uri,
                            mode: LaunchMode.externalApplication,
                          );
                        }
                      },
                      child: Text(l10n.shopPay),
                    ),
                  if (o.isPending) ...[
                    const SizedBox(height: 8),
                    OutlinedButton(
                      onPressed: () async {
                        try {
                          await orders.cancel(o.id);
                          if (!context.mounted) return;
                          toastification.show(
                            context: context,
                            type: ToastificationType.info,
                            title: Text(l10n.shopCancelled),
                            autoCloseDuration: const Duration(seconds: 2),
                          );
                        } catch (e) {
                          if (!context.mounted) return;
                          toastification.show(
                            context: context,
                            type: ToastificationType.error,
                            title: Text(e.toString()),
                            autoCloseDuration: const Duration(seconds: 3),
                          );
                        }
                      },
                      child: Text(l10n.shopCancel),
                    ),
                  ],
                ],
              );
            }),
          ),
        ],
      ),
    );
  }

  Widget _row(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 140,
            child: Text(
              label,
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }
}
