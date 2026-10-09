import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/auth_controller.dart';
import '../../l10n/app_localizations.dart';

class HomePage extends StatelessWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final auth = Get.find<AuthController>();
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(l10n.home, style: Theme.of(context).textTheme.headlineSmall),
          const SizedBox(height: 12),
          const Text(
            'Wallet, markets, and notices will use Client Center after login.',
          ),
          const SizedBox(height: 24),
          OutlinedButton.icon(
            onPressed: auth.continueWithPartner,
            icon: const Icon(Icons.link),
            label: Text(l10n.linkPartnerAccount),
          ),
        ],
      ),
    );
  }
}
