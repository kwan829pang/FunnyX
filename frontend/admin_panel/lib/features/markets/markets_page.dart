import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/markets_controller.dart';
import '../../l10n/app_localizations.dart';

class MarketsPage extends GetView<MarketsController> {
  const MarketsPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  l10n.markets,
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
              ),
              Obx(() {
                final selected = controller.statusFilter.value ?? '';
                return DropdownButton<String>(
                  value: selected,
                  items: [
                    DropdownMenuItem(
                      value: '',
                      child: Text(l10n.statusFilterAll),
                    ),
                    const DropdownMenuItem(
                      value: 'pending_locked',
                      child: Text('Pending locked'),
                    ),
                    const DropdownMenuItem(
                      value: 'active',
                      child: Text('Active'),
                    ),
                    const DropdownMenuItem(
                      value: 'rejected',
                      child: Text('Rejected'),
                    ),
                  ],
                  onChanged: (v) {
                    final filter = (v == null || v.isEmpty) ? null : v;
                    controller.setFilter(filter);
                  },
                );
              }),
              IconButton(
                onPressed: controller.reload,
                icon: const Icon(Icons.refresh),
              ),
            ],
          ),
          const SizedBox(height: 8),
          const Text(
            'Review Corp-submitted market pairs. Approve transfers locked Game Coin into the pool and activates Core Engine.',
          ),
          const SizedBox(height: 16),
          Expanded(
            child: Obx(() {
              if (controller.loading.value && controller.markets.isEmpty) {
                return const Center(child: CircularProgressIndicator());
              }
              final rows = controller.markets;
              if (rows.isEmpty) {
                return const Center(child: Text('No markets found.'));
              }
              return SingleChildScrollView(
                child: DataTable(
                  columns: [
                    const DataColumn(label: Text('ID')),
                    const DataColumn(label: Text('Market')),
                    DataColumn(label: Text(l10n.corpId)),
                    const DataColumn(label: Text('Game')),
                    const DataColumn(label: Text('Pair')),
                    const DataColumn(label: Text('Lock')),
                    const DataColumn(label: Text('Pool')),
                    DataColumn(label: Text(l10n.status)),
                    DataColumn(label: Text(l10n.actions)),
                  ],
                  rows: [
                    for (final m in rows)
                      DataRow(
                        cells: [
                          DataCell(Text('${m.id}')),
                          DataCell(Text(m.marketName)),
                          DataCell(Text('${m.corporateUserId}')),
                          DataCell(Text('${m.gameId}')),
                          DataCell(
                            Text(
                              '${m.baseGameCoin ?? m.pool?.baseAmount ?? '-'} / ${m.quoteGameCoin ?? '-'}',
                            ),
                          ),
                          DataCell(
                            Text(
                              m.lockAmount?.toStringAsFixed(2) ?? '-',
                            ),
                          ),
                          DataCell(
                            Text(
                              m.pool == null
                                  ? '-'
                                  : '${m.pool!.baseAmount.toStringAsFixed(0)} / ${m.pool!.quoteAmount.toStringAsFixed(0)}',
                            ),
                          ),
                          DataCell(Text(m.status)),
                          DataCell(
                            m.status == 'pending_locked'
                                ? Row(
                                    mainAxisSize: MainAxisSize.min,
                                    children: [
                                      TextButton(
                                        onPressed: controller.busy.value
                                            ? null
                                            : () => controller.approve(m),
                                        child: const Text('Approve'),
                                      ),
                                      TextButton(
                                        onPressed: controller.busy.value
                                            ? null
                                            : () => controller.reject(m),
                                        child: const Text('Reject'),
                                      ),
                                    ],
                                  )
                                : const Text('—'),
                          ),
                        ],
                      ),
                  ],
                ),
              );
            }),
          ),
        ],
      ),
    );
  }
}
