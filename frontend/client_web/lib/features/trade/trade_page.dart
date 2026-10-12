import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../../controllers/trade_controller.dart';
import '../../l10n/app_localizations.dart';

class TradePage extends GetView<TradeController> {
  const TradePage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final qtyCtrl = TextEditingController(text: '1');
    final priceCtrl = TextEditingController(text: '100');
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Obx(() {
        final markets = controller.markets;
        final selected = controller.selected.value;
        final accounts = controller.gameAccounts;
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    l10n.trade,
                    style: Theme.of(context).textTheme.headlineSmall,
                  ),
                ),
                IconButton(
                  onPressed: controller.reload,
                  icon: const Icon(Icons.refresh),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(l10n.tradeHint),
            const SizedBox(height: 12),
            if (controller.loading.value && markets.isEmpty)
              const LinearProgressIndicator(),
            DropdownButtonFormField<ExchangeMarket>(
              initialValue: selected,
              decoration: InputDecoration(labelText: l10n.tradeMarket),
              items: [
                for (final m in markets)
                  DropdownMenuItem(value: m, child: Text(m.symbol)),
              ],
              onChanged: (m) {
                if (m != null) controller.selectMarket(m);
              },
            ),
            const SizedBox(height: 8),
            Text(l10n.tradeBook, style: Theme.of(context).textTheme.titleSmall),
            Container(
              height: 120,
              padding: const EdgeInsets.all(8),
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              child: SingleChildScrollView(
                child: Text(controller.bookText.value),
              ),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: qtyCtrl,
                    decoration: InputDecoration(labelText: l10n.tradeQty),
                    keyboardType: TextInputType.number,
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: TextField(
                    controller: priceCtrl,
                    decoration: InputDecoration(labelText: l10n.tradePrice),
                    keyboardType: TextInputType.number,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            if (accounts.isEmpty)
              Text(l10n.shopNoGameAccount)
            else
              Row(
                children: [
                  FilledButton(
                    onPressed: () => controller.place(
                      side: 'buy',
                      orderType: 'price',
                      qtyText: qtyCtrl.text,
                      priceText: priceCtrl.text,
                      gameAccountId: accounts.first.id,
                    ),
                    child: Text(l10n.tradeBuy),
                  ),
                  const SizedBox(width: 8),
                  OutlinedButton(
                    onPressed: () => controller.place(
                      side: 'sell',
                      orderType: 'price',
                      qtyText: qtyCtrl.text,
                      priceText: priceCtrl.text,
                      gameAccountId: accounts.first.id,
                    ),
                    child: Text(l10n.tradeSell),
                  ),
                ],
              ),
            const SizedBox(height: 16),
            Text(
              l10n.tradeOpenOrders,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            Expanded(
              child: ListView(
                children: [
                  for (final o in controller.orders)
                    ListTile(
                      title: Text(
                        '#${o.id} ${o.side} ${o.quantity}@${o.price} ${o.symbol}',
                      ),
                      subtitle: Text('${o.status} · ${o.engineOrderId}'),
                      trailing: o.status == 'cancelled' || o.status == 'filled'
                          ? null
                          : TextButton(
                              onPressed: () => controller.cancel(o),
                              child: Text(l10n.tradeCancel),
                            ),
                    ),
                ],
              ),
            ),
          ],
        );
      }),
    );
  }
}
