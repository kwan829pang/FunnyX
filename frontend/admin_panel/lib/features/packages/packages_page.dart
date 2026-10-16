import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/packages_controller.dart';
import '../../l10n/app_localizations.dart';
import '../../widgets/page_frame.dart';
import '../../widgets/scrollable_data_table.dart';

class PackagesPage extends GetView<PackagesController> {
  const PackagesPage({super.key});

  Future<void> _showCreateDialog(BuildContext context) async {
    final l10n = AppLocalizations.of(context);
    final codeCtrl = TextEditingController(text: 'PLT_');
    final nameCtrl = TextEditingController();
    final coinCtrl = TextEditingController();
    final priceCtrl = TextEditingController();
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) {
        return AlertDialog(
          title: Text(l10n.packagesCreate),
          content: SizedBox(
            width: 420,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                  controller: codeCtrl,
                  decoration: InputDecoration(labelText: l10n.packagesCode),
                ),
                TextField(
                  controller: nameCtrl,
                  decoration: InputDecoration(labelText: l10n.packagesName),
                ),
                TextField(
                  controller: coinCtrl,
                  keyboardType: const TextInputType.numberWithOptions(decimal: true),
                  decoration: InputDecoration(labelText: l10n.packagesCoinAmount),
                ),
                TextField(
                  controller: priceCtrl,
                  keyboardType: const TextInputType.numberWithOptions(decimal: true),
                  decoration: InputDecoration(labelText: l10n.setupFiatPrice),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(false),
              child: Text(l10n.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.of(ctx).pop(true),
              child: Text(l10n.packagesCreate),
            ),
          ],
        );
      },
    );
    if (ok == true) {
      await controller.createPackage(
        code: codeCtrl.text,
        name: nameCtrl.text,
        coinAmountText: coinCtrl.text,
        fiatPriceText: priceCtrl.text,
      );
    }
    codeCtrl.dispose();
    nameCtrl.dispose();
    coinCtrl.dispose();
    priceCtrl.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return PageFrame(
      title: l10n.packages,
      subtitle: l10n.packagesHint,
      actions: [
        IconButton(
          onPressed: controller.reload,
          icon: const Icon(Icons.refresh),
        ),
        FilledButton.icon(
          onPressed: () => _showCreateDialog(context),
          icon: const Icon(Icons.add),
          label: Text(l10n.packagesCreate),
        ),
      ],
      child: Obx(() {
        if (controller.loading.value && controller.packages.isEmpty) {
          return const Center(child: CircularProgressIndicator());
        }
        final rows = controller.packages;
        if (rows.isEmpty) {
          return Center(child: Text(l10n.packagesEmpty));
        }
        return ScrollableDataTable(
          table: DataTable(
            columns: [
              DataColumn(label: Text(l10n.packagesCode)),
              DataColumn(label: Text(l10n.packagesName)),
              DataColumn(label: Text(l10n.packagesCoinAmount)),
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
                    DataCell(Text(p.coinAmount.toStringAsFixed(0))),
                    DataCell(
                      SizedBox(
                        width: 100,
                        child: _PriceEditor(
                          key: ValueKey('price-${p.id}-${p.fiatPrice}'),
                          initial: p.fiatPrice.toString(),
                          onSave: (v) => controller.savePrice(p, v),
                        ),
                      ),
                    ),
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
    );
  }
}

class _PriceEditor extends StatefulWidget {
  const _PriceEditor({super.key, required this.initial, required this.onSave});

  final String initial;
  final Future<void> Function(String) onSave;

  @override
  State<_PriceEditor> createState() => _PriceEditorState();
}

class _PriceEditorState extends State<_PriceEditor> {
  late final TextEditingController _ctrl;

  @override
  void initState() {
    super.initState();
    _ctrl = TextEditingController(text: widget.initial);
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: _ctrl,
      keyboardType: const TextInputType.numberWithOptions(decimal: true),
      decoration: const InputDecoration(
        isDense: true,
        border: OutlineInputBorder(),
      ),
      onSubmitted: widget.onSave,
    );
  }
}
