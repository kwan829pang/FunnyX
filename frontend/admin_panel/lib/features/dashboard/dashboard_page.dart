import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../l10n/app_localizations.dart';

class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(l10n.dashboard, style: Theme.of(context).textTheme.headlineSmall),
          const SizedBox(height: 12),
          Text(l10n.setupWizardSubtitle),
          const SizedBox(height: 16),
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              FilledButton.tonal(
                onPressed: () => Get.toNamed('/packages'),
                child: Text(l10n.packages),
              ),
              FilledButton.tonal(
                onPressed: () => Get.toNamed('/corp-products'),
                child: Text(l10n.corpProducts),
              ),
              FilledButton.tonal(
                onPressed: () => Get.toNamed('/setup'),
                child: Text(l10n.setup),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
