import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/auth_controller.dart';
import '../../controllers/wallet_controller.dart';
import '../../l10n/app_localizations.dart';

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  late final WalletController wallet;

  @override
  void initState() {
    super.initState();
    wallet = Get.find<WalletController>();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      wallet.reload();
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final auth = Get.find<AuthController>();
    return Padding(
      padding: const EdgeInsets.all(24),
      child: ListView(
        children: [
          Text(l10n.home, style: Theme.of(context).textTheme.headlineSmall),
          const SizedBox(height: 16),
          Obx(() {
            final p = wallet.profile.value;
            final name = p?.username ?? '…';
            final email = p?.email;
            return Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      l10n.walletProfile,
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text('${l10n.username}: $name'),
                    if (email != null && email.isNotEmpty)
                      Text('${l10n.walletEmail}: $email'),
                    if (p != null)
                      Text('${l10n.shopColStatus}: ${p.status}'),
                  ],
                ),
              ),
            );
          }),
          const SizedBox(height: 12),
          Obx(() {
            final plt = wallet.pltAvailable;
            final count = wallet.wallets.length;
            return Card(
              child: ListTile(
                leading: const Icon(Icons.account_balance_wallet_outlined),
                title: Text(l10n.wallet),
                subtitle: Text(
                  '${l10n.walletPltAvailable}: $plt · '
                  '${l10n.walletBalanceCount(count)}',
                ),
                trailing: const Icon(Icons.chevron_right),
                onTap: () => Get.toNamed('/wallet'),
              ),
            );
          }),
          const SizedBox(height: 24),
          Text(l10n.marketplaceEligible),
          const SizedBox(height: 16),
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              FilledButton.icon(
                onPressed: () => Get.toNamed('/wallet'),
                icon: const Icon(Icons.account_balance_wallet),
                label: Text(l10n.wallet),
              ),
              FilledButton.icon(
                onPressed: () => Get.toNamed('/shop'),
                icon: const Icon(Icons.shopping_bag),
                label: Text(l10n.shop),
              ),
              FilledButton.icon(
                onPressed: () => Get.toNamed('/marketplace'),
                icon: const Icon(Icons.storefront),
                label: Text(l10n.marketplace),
              ),
              OutlinedButton.icon(
                onPressed: () => Get.toNamed('/corp/register'),
                icon: const Icon(Icons.business),
                label: Text(l10n.corpRegisterTitle),
              ),
              OutlinedButton.icon(
                onPressed: auth.continueWithPartner,
                icon: const Icon(Icons.link),
                label: Text(l10n.linkPartnerAccount),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
