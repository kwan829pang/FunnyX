import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../../controllers/fees_controller.dart';
import '../../l10n/app_localizations.dart';

class FeesPage extends GetView<FeesController> {
  const FeesPage({super.key});

  Future<void> _createFee(BuildContext context) async {
    final l10n = AppLocalizations.of(context);
    final corps = controller.corps;
    if (corps.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('No corporate users (POSTGRES_URL required)')),
      );
      return;
    }
    var corpId = corps.first.id;
    var feeType = 'year1_package';
    final amountCtrl = TextEditingController(
      text: (controller.catalog['year1_package'] ?? 80000).toStringAsFixed(0),
    );
    var currency = 'HKD';
    final yearCtrl = TextEditingController(
      text: DateTime.now().year.toString(),
    );

    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) {
        return StatefulBuilder(
          builder: (ctx, setState) {
            return AlertDialog(
              title: const Text('Create platform fee'),
              content: SizedBox(
                width: 420,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    DropdownButtonFormField<int>(
                      initialValue: corpId,
                      decoration: const InputDecoration(labelText: 'Corporate user'),
                      items: [
                        for (final c in corps)
                          DropdownMenuItem(
                            value: c.id,
                            child: Text('#${c.id} ${c.companyName}'),
                          ),
                      ],
                      onChanged: (v) {
                        if (v != null) setState(() => corpId = v);
                      },
                    ),
                    DropdownButtonFormField<String>(
                      initialValue: feeType,
                      decoration: const InputDecoration(labelText: 'Fee type'),
                      items: const [
                        DropdownMenuItem(
                          value: 'year1_package',
                          child: Text('Year-1 package (80K)'),
                        ),
                        DropdownMenuItem(
                          value: 'license_renewal',
                          child: Text('License renewal (10K)'),
                        ),
                        DropdownMenuItem(
                          value: 'new_pair',
                          child: Text('Extra pair (10K)'),
                        ),
                        DropdownMenuItem(
                          value: 'supply_increase',
                          child: Text('Supply increase'),
                        ),
                      ],
                      onChanged: (v) {
                        if (v == null) return;
                        setState(() {
                          feeType = v;
                          final suggested = controller.catalog[v];
                          if (suggested != null) {
                            amountCtrl.text = suggested.toStringAsFixed(0);
                          }
                        });
                      },
                    ),
                    TextField(
                      controller: amountCtrl,
                      keyboardType: TextInputType.number,
                      decoration: const InputDecoration(labelText: 'Amount'),
                    ),
                    DropdownButtonFormField<String>(
                      initialValue: currency,
                      decoration: const InputDecoration(labelText: 'Pay currency'),
                      items: const [
                        DropdownMenuItem(value: 'HKD', child: Text('HKD')),
                        DropdownMenuItem(value: 'USD', child: Text('USD')),
                        DropdownMenuItem(value: 'USDT', child: Text('USDT')),
                      ],
                      onChanged: (v) {
                        if (v != null) setState(() => currency = v);
                      },
                    ),
                    TextField(
                      controller: yearCtrl,
                      keyboardType: TextInputType.number,
                      decoration: const InputDecoration(labelText: 'License year'),
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
                  child: const Text('Create'),
                ),
              ],
            );
          },
        );
      },
    );
    if (ok == true) {
      final amount = double.tryParse(amountCtrl.text.trim()) ?? 0;
      final year = int.tryParse(yearCtrl.text.trim());
      await controller.createFee(
        corporateUserId: corpId,
        feeType: feeType,
        amount: amount,
        payCurrency: currency,
        licenseYear: year,
      );
    }
    amountCtrl.dispose();
    yearCtrl.dispose();
  }

  Future<void> _editDeadline(BuildContext context, AdminFeeCorp corp) async {
    final l10n = AppLocalizations.of(context);
    final daysCtrl = TextEditingController(text: '365');
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) {
        return AlertDialog(
          title: Text('License deadline — ${corp.companyName}'),
          content: TextField(
            controller: daysCtrl,
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(
              labelText: 'Days from now (0 = clear / unlimited)',
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(false),
              child: Text(l10n.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.of(ctx).pop(true),
              child: const Text('Save'),
            ),
          ],
        );
      },
    );
    if (ok == true) {
      final days = int.tryParse(daysCtrl.text.trim()) ?? 0;
      final deadline = days <= 0
          ? 0
          : DateTime.now()
              .add(Duration(days: days))
              .millisecondsSinceEpoch;
      await controller.setLicenseDeadline(corp, deadline);
    }
    daysCtrl.dispose();
  }

  String _fmtDeadline(int ms) {
    if (ms <= 0) return 'unlimited';
    return DateTime.fromMillisecondsSinceEpoch(ms).toUtc().toIso8601String();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  'Fees & license',
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
              ),
              FilledButton.icon(
                onPressed: () => _createFee(context),
                icon: const Icon(Icons.add),
                label: const Text('Create fee'),
              ),
              IconButton(
                onPressed: controller.reload,
                icon: const Icon(Icons.refresh),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Obx(() {
            final c = controller.catalog;
            final y1 = c['year1_package'] ?? 80000;
            final ren = c['license_renewal'] ?? 10000;
            final pair = c['new_pair'] ?? 10000;
            return Text(
              'C6/C9 catalog: Year-1 $y1 / renewal $ren / extra pair $pair (HKD/USD/USDT). '
              'Disable api_enabled to block Corp MasterSigned (including new markets).',
            );
          }),
          const SizedBox(height: 16),
          Expanded(
            child: Obx(() {
              if (controller.loading.value &&
                  controller.fees.isEmpty &&
                  controller.corps.isEmpty) {
                return const Center(child: CircularProgressIndicator());
              }
              return ListView(
                children: [
                  Text(
                    'Corporate license',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 8),
                  if (controller.corps.isEmpty)
                    const Text('No corps loaded (set POSTGRES_URL).')
                  else
                    SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      child: DataTable(
                        columns: const [
                          DataColumn(label: Text('ID')),
                          DataColumn(label: Text('Company')),
                          DataColumn(label: Text('Status')),
                          DataColumn(label: Text('API')),
                          DataColumn(label: Text('License deadline')),
                          DataColumn(label: Text('Actions')),
                        ],
                        rows: [
                          for (final c in controller.corps)
                            DataRow(
                              cells: [
                                DataCell(Text('${c.id}')),
                                DataCell(Text(c.companyName)),
                                DataCell(Text(c.status)),
                                DataCell(Text(c.apiEnabled ? 'on' : 'off')),
                                DataCell(Text(_fmtDeadline(c.licenseDeadlineAt))),
                                DataCell(
                                  Row(
                                    children: [
                                      TextButton(
                                        onPressed: () => controller.setApiEnabled(
                                          c,
                                          !c.apiEnabled,
                                        ),
                                        child: Text(
                                          c.apiEnabled ? 'Disable' : 'Enable',
                                        ),
                                      ),
                                      TextButton(
                                        onPressed: () => _editDeadline(context, c),
                                        child: const Text('Deadline'),
                                      ),
                                    ],
                                  ),
                                ),
                              ],
                            ),
                        ],
                      ),
                    ),
                  const SizedBox(height: 24),
                  Text(
                    'Platform fees',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 8),
                  if (controller.fees.isEmpty)
                    const Text('No fee rows yet.')
                  else
                    SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      child: DataTable(
                        columns: const [
                          DataColumn(label: Text('ID')),
                          DataColumn(label: Text('Corp')),
                          DataColumn(label: Text('Type')),
                          DataColumn(label: Text('Amount')),
                          DataColumn(label: Text('Currency')),
                          DataColumn(label: Text('Status')),
                          DataColumn(label: Text('Year')),
                        ],
                        rows: [
                          for (final f in controller.fees)
                            DataRow(
                              cells: [
                                DataCell(Text('${f.id}')),
                                DataCell(Text('${f.corporateUserId}')),
                                DataCell(Text(f.feeType)),
                                DataCell(Text(f.amount.toString())),
                                DataCell(Text(f.payCurrency)),
                                DataCell(Text(f.status)),
                                DataCell(Text('${f.licenseYear}')),
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
  }
}
