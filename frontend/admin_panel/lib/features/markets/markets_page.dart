import 'package:flutter/material.dart';

import '../../l10n/app_localizations.dart';

class MarketsPage extends StatelessWidget {
  const MarketsPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(l10n.markets, style: Theme.of(context).textTheme.headlineSmall),
          const SizedBox(height: 12),
          const Text('Market list will use a server-side table once Admin API exists.'),
        ],
      ),
    );
  }
}
