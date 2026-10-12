import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../../controllers/wallet_controller.dart';
import '../../l10n/app_localizations.dart';

class WalletPage extends GetView<WalletController> {
  const WalletPage({super.key});

  Future<void> _showMoneyDialog(
    BuildContext context, {
    required bool deposit,
  }) async {
    final l10n = AppLocalizations.of(context);
    final accounts = controller.gameAccounts;
    if (accounts.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(l10n.shopNoGameAccount)),
      );
      return;
    }
    var selected = accounts.first;
    final coinCtrl = TextEditingController(text: 'PLT');
    final amountCtrl = TextEditingController();
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) {
        return StatefulBuilder(
          builder: (ctx, setLocal) {
            return AlertDialog(
              title: Text(deposit ? l10n.walletDeposit : l10n.walletWithdraw),
              content: SizedBox(
                width: 420,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    InputDecorator(
                      decoration: InputDecoration(labelText: l10n.shopGameAccount),
                      child: DropdownButtonHideUnderline(
                        child: DropdownButton<GameAccountBinding>(
                          isExpanded: true,
                          value: selected,
                          items: [
                            for (final a in accounts)
                              DropdownMenuItem(value: a, child: Text(a.label)),
                          ],
                          onChanged: (v) {
                            if (v != null) setLocal(() => selected = v);
                          },
                        ),
                      ),
                    ),
                    TextField(
                      controller: coinCtrl,
                      decoration: InputDecoration(labelText: l10n.walletGameCoin),
                    ),
                    TextField(
                      controller: amountCtrl,
                      keyboardType: const TextInputType.numberWithOptions(
                        decimal: true,
                      ),
                      decoration: InputDecoration(labelText: l10n.walletAmount),
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(ctx).pop(false),
                  child: Text(l10n.shopCancel),
                ),
                FilledButton(
                  onPressed: () => Navigator.of(ctx).pop(true),
                  child: Text(deposit ? l10n.walletDeposit : l10n.walletWithdraw),
                ),
              ],
            );
          },
        );
      },
    );
    if (ok == true) {
      if (deposit) {
        await controller.deposit(
          gameAccountId: selected.id,
          gameCoin: coinCtrl.text,
          amountText: amountCtrl.text,
        );
      } else {
        await controller.withdraw(
          gameAccountId: selected.id,
          gameCoin: coinCtrl.text,
          amountText: amountCtrl.text,
        );
      }
    }
    coinCtrl.dispose();
    amountCtrl.dispose();
  }

  String _fmtTime(int ms) {
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
    return ResponsiveScope(
      builder: (context, size) {
        final mobile = size == AppLayoutSize.mobile;
        return Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                children: [
                  Expanded(
                    child: Text(
                      l10n.wallet,
                      style: Theme.of(context).textTheme.headlineSmall,
                    ),
                  ),
                  IconButton(
                    onPressed: controller.reload,
                    icon: const Icon(Icons.refresh),
                  ),
                  OutlinedButton(
                    onPressed: () => _showMoneyDialog(context, deposit: false),
                    child: Text(l10n.walletWithdraw),
                  ),
                  const SizedBox(width: 8),
                  FilledButton(
                    onPressed: () => _showMoneyDialog(context, deposit: true),
                    child: Text(l10n.walletDeposit),
                  ),
                ],
              ),
              const SizedBox(height: 8),
              Text(l10n.walletHint),
              const SizedBox(height: 16),
              Expanded(
                child: Obx(() {
                  if (controller.loading.value &&
                      controller.wallets.isEmpty &&
                      controller.transactions.isEmpty) {
                    return const Center(child: CircularProgressIndicator());
                  }
                  return ListView(
                    children: [
                      Text(
                        l10n.walletBalances,
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      const SizedBox(height: 8),
                      if (controller.wallets.isEmpty)
                        Text(l10n.walletEmptyBalances)
                      else if (mobile)
                        ...[
                          for (final w in controller.wallets)
                            Card(
                              child: ListTile(
                                title: Text('${w.gameCoin} · ${w.gameCode}'),
                                subtitle: Text(
                                  '${w.gameAccountId}\n'
                                  '${l10n.walletAvailable}: ${w.available} · '
                                  '${l10n.walletLocked}: ${w.locked}',
                                ),
                                isThreeLine: true,
                              ),
                            ),
                        ]
                      else
                        SingleChildScrollView(
                          scrollDirection: Axis.horizontal,
                          child: DataTable(
                            columns: [
                              DataColumn(label: Text(l10n.walletGameCoin)),
                              DataColumn(label: Text(l10n.marketplaceGame)),
                              DataColumn(label: Text(l10n.shopGameAccount)),
                              DataColumn(label: Text(l10n.walletAvailable)),
                              DataColumn(label: Text(l10n.walletLocked)),
                            ],
                            rows: [
                              for (final w in controller.wallets)
                                DataRow(
                                  cells: [
                                    DataCell(Text(w.gameCoin)),
                                    DataCell(Text(w.gameCode)),
                                    DataCell(Text(w.gameAccountId)),
                                    DataCell(Text(w.available.toString())),
                                    DataCell(Text(w.locked.toString())),
                                  ],
                                ),
                            ],
                          ),
                        ),
                      const SizedBox(height: 24),
                      Text(
                        l10n.walletTransactions,
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      const SizedBox(height: 8),
                      if (controller.transactions.isEmpty)
                        Text(l10n.walletEmptyTxns)
                      else if (mobile)
                        ...[
                          for (final t in controller.transactions)
                            Card(
                              child: ListTile(
                                title: Text(
                                  '${t.transactionType} ${t.amount} '
                                  '${t.gameCoin ?? ''}',
                                ),
                                subtitle: Text(
                                  '#${t.id} · ${t.status}\n${_fmtTime(t.createdAt)}',
                                ),
                                isThreeLine: true,
                              ),
                            ),
                        ]
                      else
                        SingleChildScrollView(
                          scrollDirection: Axis.horizontal,
                          child: DataTable(
                            columns: [
                              DataColumn(label: Text(l10n.walletTxnId)),
                              DataColumn(label: Text(l10n.walletTxnType)),
                              DataColumn(label: Text(l10n.walletAmount)),
                              DataColumn(label: Text(l10n.walletGameCoin)),
                              DataColumn(label: Text(l10n.shopColStatus)),
                              DataColumn(label: Text(l10n.shopCreatedAt)),
                            ],
                            rows: [
                              for (final t in controller.transactions)
                                DataRow(
                                  cells: [
                                    DataCell(Text('${t.id}')),
                                    DataCell(Text(t.transactionType)),
                                    DataCell(Text(t.amount.toString())),
                                    DataCell(Text(t.gameCoin ?? '—')),
                                    DataCell(Text(t.status)),
                                    DataCell(Text(_fmtTime(t.createdAt))),
                                  ],
                                ),
                            ],
                          ),
                        ),
                    ],
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
