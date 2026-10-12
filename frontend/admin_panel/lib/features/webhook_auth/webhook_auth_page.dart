import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../../controllers/webhook_auth_controller.dart';
import '../../l10n/app_localizations.dart';

class WebhookAuthPage extends GetView<WebhookAuthController> {
  const WebhookAuthPage({super.key});

  Future<void> _showForm(
    BuildContext context, {
    AdminWebhookPostAuth? existing,
  }) async {
    final l10n = AppLocalizations.of(context);
    final sourceCtrl = TextEditingController(text: existing?.sourceCode ?? '');
    final domainCtrl = TextEditingController(text: existing?.domainPattern ?? '');
    final suffixCtrl = TextEditingController(text: existing?.authKeySuffix ?? '');
    final ppkCtrl = TextEditingController(text: existing?.ppk ?? '');
    final ipsCtrl = TextEditingController(text: existing?.allowedIps ?? '');
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) {
        return AlertDialog(
          title: Text(existing == null ? 'Create Post_Auth' : 'Edit Post_Auth'),
          content: SizedBox(
            width: 480,
            child: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  TextField(
                    controller: sourceCtrl,
                    decoration: const InputDecoration(labelText: 'source_code'),
                  ),
                  TextField(
                    controller: domainCtrl,
                    decoration: const InputDecoration(
                      labelText: 'domain_pattern',
                      hintText: 'exact.host or *.suffix',
                    ),
                  ),
                  TextField(
                    controller: suffixCtrl,
                    decoration: const InputDecoration(
                      labelText: 'auth_key_suffix',
                      hintText: 'XXX in Post_Auth_XXX',
                    ),
                  ),
                  TextField(
                    controller: ppkCtrl,
                    decoration: const InputDecoration(
                      labelText: 'ppk (optional)',
                    ),
                  ),
                  TextField(
                    controller: ipsCtrl,
                    decoration: const InputDecoration(
                      labelText: 'allowed_ips',
                      hintText: 'comma or pipe separated',
                    ),
                  ),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(false),
              child: Text(l10n.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.of(ctx).pop(true),
              child: Text(existing == null ? 'Create' : 'Save'),
            ),
          ],
        );
      },
    );
    if (ok == true) {
      if (existing == null) {
        await controller.createItem(
          sourceCode: sourceCtrl.text,
          domainPattern: domainCtrl.text,
          authKeySuffix: suffixCtrl.text,
          ppk: ppkCtrl.text,
          allowedIps: ipsCtrl.text,
        );
      } else {
        await controller.updateItem(
          id: existing.id,
          sourceCode: sourceCtrl.text,
          domainPattern: domainCtrl.text,
          authKeySuffix: suffixCtrl.text,
          ppk: ppkCtrl.text,
          allowedIps: ipsCtrl.text,
        );
      }
    }
    sourceCtrl.dispose();
    domainCtrl.dispose();
    suffixCtrl.dispose();
    ppkCtrl.dispose();
    ipsCtrl.dispose();
  }

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
                  'Webhook Post_Auth',
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
              ),
              IconButton(
                onPressed: controller.reload,
                icon: const Icon(Icons.refresh),
              ),
              FilledButton.icon(
                onPressed: () => _showForm(context),
                icon: const Icon(Icons.add),
                label: const Text('Create'),
              ),
            ],
          ),
          const SizedBox(height: 8),
          const Text(
            'Inbound partner auth mapping (source / domain / IP / auth key).',
          ),
          const SizedBox(height: 16),
          Expanded(
            child: Obx(() {
              if (controller.loading.value && controller.items.isEmpty) {
                return const Center(child: CircularProgressIndicator());
              }
              final rows = controller.items;
              if (rows.isEmpty) {
                return const Center(child: Text('No Post_Auth rows'));
              }
              return SingleChildScrollView(
                child: DataTable(
                  columns: [
                    const DataColumn(label: Text('source_code')),
                    const DataColumn(label: Text('domain')),
                    const DataColumn(label: Text('suffix')),
                    const DataColumn(label: Text('ppk')),
                    const DataColumn(label: Text('allowed_ips')),
                    DataColumn(label: Text(l10n.status)),
                    DataColumn(label: Text(l10n.actions)),
                  ],
                  rows: [
                    for (final row in rows)
                      DataRow(
                        cells: [
                          DataCell(Text(row.sourceCode)),
                          DataCell(Text(row.domainPattern)),
                          DataCell(Text(row.authKeySuffix)),
                          DataCell(Text(row.ppk ?? '—')),
                          DataCell(Text(row.allowedIps)),
                          DataCell(Text(row.status)),
                          DataCell(
                            Row(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                IconButton(
                                  tooltip: 'Edit',
                                  onPressed: () =>
                                      _showForm(context, existing: row),
                                  icon: const Icon(Icons.edit_outlined),
                                ),
                                PopupMenuButton<String>(
                                  onSelected: (s) =>
                                      controller.setStatus(row, s),
                                  itemBuilder: (_) => [
                                    PopupMenuItem(
                                      value: 'active',
                                      child: Text(l10n.statusActive),
                                    ),
                                    PopupMenuItem(
                                      value: 'inactive',
                                      child: Text(l10n.statusInactive),
                                    ),
                                  ],
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
