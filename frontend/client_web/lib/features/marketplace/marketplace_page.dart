import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/marketplace_controller.dart';
import '../../l10n/app_localizations.dart';

class MarketplacePage extends StatefulWidget {
  const MarketplacePage({super.key});

  @override
  State<MarketplacePage> createState() => _MarketplacePageState();
}

class _MarketplacePageState extends State<MarketplacePage> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      Get.find<MarketplaceController>().loadList();
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final ctrl = Get.find<MarketplaceController>();

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  l10n.marketplace,
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
              ),
              Obx(() {
                final can = ctrl.eligibility.value?.canPlace == true;
                return FilledButton.icon(
                  onPressed: can ? () => Get.toNamed('/marketplace/new') : null,
                  icon: const Icon(Icons.add),
                  label: Text(l10n.marketplacePlaceDeal),
                );
              }),
            ],
          ),
          const SizedBox(height: 12),
          Obx(() {
            final elig = ctrl.eligibility.value;
            if (elig == null) return const SizedBox.shrink();
            return Card(
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      elig.canPlace
                          ? l10n.marketplaceEligible
                          : l10n.marketplaceNotEligible,
                      style: Theme.of(context).textTheme.titleSmall,
                    ),
                    const SizedBox(height: 4),
                    Text(l10n.marketplaceListingFee(elig.listingFeePlt.toStringAsFixed(0))),
                    if (elig.reasons.isNotEmpty) ...[
                      const SizedBox(height: 8),
                      Wrap(
                        spacing: 8,
                        children: elig.reasons
                            .map((r) => Chip(label: Text(_reasonLabel(l10n, r))))
                            .toList(),
                      ),
                    ],
                  ],
                ),
              ),
            );
          }),
          const SizedBox(height: 12),
          Expanded(
            child: Obx(() {
              if (ctrl.loading.value && ctrl.deals.isEmpty) {
                return const Center(child: CircularProgressIndicator());
              }
              if (ctrl.error.value != null && ctrl.deals.isEmpty) {
                return Center(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(ctrl.error.value!),
                      TextButton(
                        onPressed: ctrl.loadList,
                        child: Text(l10n.shopRetry),
                      ),
                    ],
                  ),
                );
              }
              if (ctrl.deals.isEmpty) {
                return Center(child: Text(l10n.marketplaceEmpty));
              }
              return ListView.separated(
                itemCount: ctrl.deals.length,
                separatorBuilder: (_, _) => const Divider(height: 1),
                itemBuilder: (context, i) {
                  final d = ctrl.deals[i];
                  return ListTile(
                    title: Text('${d.gameCode} · ${d.offerLabel}'),
                    subtitle: Text(
                      '${l10n.marketplaceWant}: ${d.wantLabel} · ${d.status}',
                    ),
                    trailing: const Icon(Icons.chevron_right),
                    onTap: () => Get.toNamed('/marketplace/deals/${d.id}'),
                  );
                },
              );
            }),
          ),
        ],
      ),
    );
  }

  String _reasonLabel(AppLocalizations l10n, String code) {
    return switch (code) {
      'buy_platform_token' => l10n.marketplaceGatePlt,
      'bind_listed_game' => l10n.marketplaceGateGame,
      'partner_transfer_required' => l10n.marketplaceGateTransfer,
      _ => code,
    };
  }
}
