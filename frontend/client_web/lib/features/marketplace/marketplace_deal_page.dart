import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import '../../controllers/marketplace_controller.dart';
import '../../l10n/app_localizations.dart';

class MarketplaceDealPage extends StatefulWidget {
  const MarketplaceDealPage({super.key, required this.dealId});

  final int dealId;

  @override
  State<MarketplaceDealPage> createState() => _MarketplaceDealPageState();
}

class _MarketplaceDealPageState extends State<MarketplaceDealPage> {
  GameAccountBinding? _takerAccount;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      final ctrl = Get.find<MarketplaceController>();
      await ctrl.loadDeal(widget.dealId);
      if (ctrl.gameAccounts.isNotEmpty) {
        final deal = ctrl.selectedDeal.value;
        final match = ctrl.gameAccounts
            .where((a) => deal == null || a.gameId == deal.gameId)
            .toList();
        setState(() => _takerAccount = match.isNotEmpty ? match.first : ctrl.gameAccounts.first);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final ctrl = Get.find<MarketplaceController>();

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Obx(() {
        if (ctrl.loading.value && ctrl.selectedDeal.value == null) {
          return const Center(child: CircularProgressIndicator());
        }
        final deal = ctrl.selectedDeal.value;
        if (deal == null) {
          return Center(child: Text(ctrl.error.value ?? l10n.marketplaceDealNotFound));
        }
        final isPoster = ctrl.gameAccounts.any(
          (a) => a.id == deal.posterGameAccountId || a.endUserId == deal.posterEndUserId,
        );
        final canTake = deal.status == 'open' && !isPoster;
        final canCancel = deal.status == 'open' && isPoster;

        return ListView(
          children: [
            Text(
              l10n.marketplaceDealTitle(deal.id),
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const SizedBox(height: 12),
            _row(l10n.marketplaceGame, deal.gameCode),
            _row(l10n.marketplaceOffer, deal.offerLabel),
            _row(l10n.marketplaceWant, deal.wantLabel),
            _row(l10n.shopColStatus, deal.status),
            _row(l10n.marketplaceListingFeeLabel, '${deal.listingFeePlt} PLT'),
            if (ctrl.error.value != null) ...[
              const SizedBox(height: 8),
              Text(
                ctrl.error.value!,
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ],
            const SizedBox(height: 24),
            if (canTake) ...[
              DropdownButtonFormField<GameAccountBinding>(
                key: ValueKey(_takerAccount?.id ?? 'taker'),
                initialValue: _takerAccount,
                decoration: InputDecoration(labelText: l10n.marketplaceYourGameAccount),
                items: ctrl.gameAccounts
                    .where((a) => a.gameId == deal.gameId)
                    .map(
                      (a) => DropdownMenuItem(
                        value: a,
                        child: Text('${a.gameCode} · ${a.gameAccountId}'),
                      ),
                    )
                    .toList(),
                onChanged: (v) => setState(() => _takerAccount = v),
              ),
              const SizedBox(height: 12),
              FilledButton(
                onPressed: ctrl.loading.value ? null : () => _take(ctrl, l10n),
                child: Text(l10n.marketplaceTakeDeal),
              ),
            ],
            if (canCancel) ...[
              OutlinedButton(
                onPressed: ctrl.loading.value ? null : () => _cancel(ctrl, l10n),
                child: Text(l10n.marketplaceCancelDeal),
              ),
            ],
            const SizedBox(height: 12),
            TextButton(
              onPressed: () => Get.back(),
              child: Text(l10n.marketplaceBack),
            ),
          ],
        );
      }),
    );
  }

  Widget _row(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(width: 140, child: Text(label, style: const TextStyle(fontWeight: FontWeight.w600))),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }

  Future<void> _take(MarketplaceController ctrl, AppLocalizations l10n) async {
    final account = _takerAccount;
    if (account == null) return;
    final ok = await ctrl.take(
      dealId: widget.dealId,
      requesterGameAccountId: account.id,
    );
    if (!mounted) return;
    toastification.show(
      context: context,
      type: ok ? ToastificationType.success : ToastificationType.error,
      title: Text(ok ? l10n.marketplaceTaken : (ctrl.error.value ?? l10n.marketplaceTakeFailed)),
      autoCloseDuration: const Duration(seconds: 3),
    );
  }

  Future<void> _cancel(MarketplaceController ctrl, AppLocalizations l10n) async {
    final ok = await ctrl.cancel(widget.dealId);
    if (!mounted) return;
    toastification.show(
      context: context,
      type: ok ? ToastificationType.success : ToastificationType.error,
      title: Text(ok ? l10n.marketplaceCancelled : (ctrl.error.value ?? 'error')),
      autoCloseDuration: const Duration(seconds: 3),
    );
  }
}
