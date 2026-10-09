import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/corp_products_controller.dart';
import '../../l10n/app_localizations.dart';

class CorpProductsPage extends GetView<CorpProductsController> {
  const CorpProductsPage({super.key});

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
                  l10n.corpProducts,
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
                    DropdownMenuItem(
                      value: 'draft',
                      child: Text(l10n.statusDraft),
                    ),
                    DropdownMenuItem(
                      value: 'active',
                      child: Text(l10n.statusActive),
                    ),
                    DropdownMenuItem(
                      value: 'inactive',
                      child: Text(l10n.statusInactive),
                    ),
                    DropdownMenuItem(
                      value: 'archived',
                      child: Text(l10n.statusArchived),
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
          Text(l10n.corpProductsHint),
          const SizedBox(height: 16),
          Expanded(
            child: Obx(() {
              if (controller.loading.value && controller.products.isEmpty) {
                return const Center(child: CircularProgressIndicator());
              }
              final rows = controller.products;
              if (rows.isEmpty) {
                return Center(child: Text(l10n.corpProductsEmpty));
              }
              return SingleChildScrollView(
                child: DataTable(
                  columns: [
                    DataColumn(label: Text(l10n.packagesCode)),
                    DataColumn(label: Text(l10n.packagesName)),
                    DataColumn(label: Text(l10n.corpId)),
                    DataColumn(label: Text(l10n.productType)),
                    DataColumn(label: Text(l10n.setupFiatPrice)),
                    DataColumn(label: Text(l10n.status)),
                    DataColumn(label: Text(l10n.actions)),
                  ],
                  rows: [
                    for (final p in rows)
                      DataRow(
                        cells: [
                          DataCell(Text(p.code)),
                          DataCell(Text(p.name)),
                          DataCell(Text('${p.corporateUserId}')),
                          DataCell(Text(p.productType)),
                          DataCell(Text(p.fiatPrice.toStringAsFixed(2))),
                          DataCell(Text(p.status)),
                          DataCell(
                            PopupMenuButton<String>(
                              onSelected: (s) => controller.setStatus(p, s),
                              itemBuilder: (_) => [
                                PopupMenuItem(
                                  value: 'active',
                                  child: Text(l10n.statusActive),
                                ),
                                PopupMenuItem(
                                  value: 'inactive',
                                  child: Text(l10n.statusInactive),
                                ),
                                PopupMenuItem(
                                  value: 'archived',
                                  child: Text(l10n.statusArchived),
                                ),
                              ],
                            ),
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
