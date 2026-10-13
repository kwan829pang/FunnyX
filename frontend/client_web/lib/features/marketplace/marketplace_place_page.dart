import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import '../../controllers/marketplace_controller.dart';
import '../../l10n/app_localizations.dart';

class MarketplacePlacePage extends StatefulWidget {
  const MarketplacePlacePage({super.key});

  @override
  State<MarketplacePlacePage> createState() => _MarketplacePlacePageState();
}

class _MarketplacePlacePageState extends State<MarketplacePlacePage> {
  GameAccountBinding? _account;
  String _offerType = 'game_coin';
  String _wantType = 'platform_token';
  final _offerAmount = TextEditingController(text: '1');
  final _wantAmount = TextEditingController(text: '10');
  final _offerCoinId = TextEditingController(text: '2');
  final _wantCoinId = TextEditingController(text: '1');
  String? _itemRefId;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      final ctrl = Get.find<MarketplaceController>();
      if (ctrl.gameAccounts.isEmpty) await ctrl.loadList();
      if (ctrl.gameAccounts.isNotEmpty) {
        setState(() => _account = ctrl.gameAccounts.first);
      }
    });
  }

  @override
  void dispose() {
    _offerAmount.dispose();
    _wantAmount.dispose();
    _offerCoinId.dispose();
    _wantCoinId.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final ctrl = Get.find<MarketplaceController>();
    final fee = ctrl.eligibility.value?.listingFeePlt ?? 1;

    return Padding(
      padding: const EdgeInsets.all(16),
      child: ListView(
        children: [
          Text(
            l10n.marketplacePlaceDeal,
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          const SizedBox(height: 8),
          Text(l10n.marketplaceListingFee(fee.toStringAsFixed(0))),
          const SizedBox(height: 16),
          Obx(() {
            final accounts = ctrl.gameAccounts;
            return DropdownButtonFormField<GameAccountBinding>(
              key: ValueKey(_account?.id ?? 'account'),
              initialValue: _account,
              decoration: InputDecoration(labelText: l10n.shopGameAccount),
              items: accounts
                  .map(
                    (a) => DropdownMenuItem(
                      value: a,
                      child: Text('${a.gameCode} · ${a.gameAccountId}'),
                    ),
                  )
                  .toList(),
              onChanged: (v) async {
                setState(() {
                  _account = v;
                  _itemRefId = null;
                });
                if (v != null && _offerType == 'game_item') {
                  await ctrl.loadItems(v.id);
                }
              },
            );
          }),
          const SizedBox(height: 12),
          DropdownButtonFormField<String>(
            key: ValueKey('offer_$_offerType'),
            initialValue: _offerType,
            decoration: InputDecoration(labelText: l10n.marketplaceOfferType),
            items: [
              DropdownMenuItem(value: 'game_coin', child: Text(l10n.marketplaceOfferGameCoin)),
              DropdownMenuItem(value: 'company_coin', child: Text(l10n.marketplaceOfferCompanyCoin)),
              DropdownMenuItem(value: 'game_item', child: Text(l10n.marketplaceOfferGameItem)),
            ],
            onChanged: (v) async {
              if (v == null) return;
              setState(() => _offerType = v);
              if (v == 'game_item' && _account != null) {
                await ctrl.loadItems(_account!.id);
              }
            },
          ),
          const SizedBox(height: 12),
          if (_offerType == 'game_item')
            Obx(() {
              return DropdownButtonFormField<String>(
                key: ValueKey(_itemRefId ?? 'item'),
                initialValue: _itemRefId,
                decoration: InputDecoration(labelText: l10n.marketplaceItemRef),
                items: ctrl.items
                    .map(
                      (i) => DropdownMenuItem(
                        value: i.itemRefId,
                        child: Text('${i.name} (${i.itemRefId})'),
                      ),
                    )
                    .toList(),
                onChanged: (v) => setState(() => _itemRefId = v),
              );
            })
          else
            TextField(
              controller: _offerCoinId,
              decoration: InputDecoration(labelText: l10n.marketplaceOfferCoinId),
              keyboardType: TextInputType.number,
            ),
          const SizedBox(height: 12),
          TextField(
            controller: _offerAmount,
            decoration: InputDecoration(labelText: l10n.marketplaceOfferAmount),
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
          ),
          const SizedBox(height: 12),
          DropdownButtonFormField<String>(
            key: ValueKey('want_$_wantType'),
            initialValue: _wantType,
            decoration: InputDecoration(labelText: l10n.marketplaceWantType),
            items: [
              DropdownMenuItem(value: 'platform_token', child: Text(l10n.marketplaceWantPlt)),
              DropdownMenuItem(value: 'company_coin', child: Text(l10n.marketplaceWantCompanyCoin)),
            ],
            onChanged: (v) {
              if (v == null) return;
              setState(() {
                _wantType = v;
                if (v == 'platform_token') _wantCoinId.text = '1';
              });
            },
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _wantCoinId,
            decoration: InputDecoration(labelText: l10n.marketplaceWantCoinId),
            keyboardType: TextInputType.number,
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _wantAmount,
            decoration: InputDecoration(labelText: l10n.marketplaceWantAmount),
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
          ),
          const SizedBox(height: 24),
          Obx(
            () => FilledButton(
              onPressed: ctrl.loading.value ? null : _submit,
              child: ctrl.loading.value
                  ? const SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : Text(l10n.marketplacePlaceDeal),
            ),
          ),
          Obx(() {
            final err = ctrl.error.value;
            if (err == null) return const SizedBox.shrink();
            return Padding(
              padding: const EdgeInsets.only(top: 12),
              child: Text(err, style: TextStyle(color: Theme.of(context).colorScheme.error)),
            );
          }),
        ],
      ),
    );
  }

  Future<void> _submit() async {
    final l10n = AppLocalizations.of(context);
    final ctrl = Get.find<MarketplaceController>();
    final account = _account;
    if (account == null) {
      toastification.show(
        context: context,
        type: ToastificationType.warning,
        title: Text(l10n.shopNoGameAccount),
        autoCloseDuration: const Duration(seconds: 3),
      );
      return;
    }
    final deal = await ctrl.place(
      posterGameAccountId: account.id,
      offerAssetType: _offerType,
      offerGameCoinId: _offerType == 'game_item' ? null : int.tryParse(_offerCoinId.text),
      offerItemRefId: _offerType == 'game_item' ? _itemRefId : null,
      offerAmount: double.tryParse(_offerAmount.text) ?? 0,
      wantAssetType: _wantType,
      wantGameCoinId: int.tryParse(_wantCoinId.text) ?? 1,
      wantAmount: double.tryParse(_wantAmount.text) ?? 0,
    );
    if (deal != null && mounted) {
      toastification.show(
        context: context,
        type: ToastificationType.success,
        title: Text(l10n.marketplacePlaced),
        autoCloseDuration: const Duration(seconds: 3),
      );
      Get.offNamed('/marketplace/deals/${deal.id}');
    }
  }
}
