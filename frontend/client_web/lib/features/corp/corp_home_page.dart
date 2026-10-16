import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/corp_controller.dart';
import '../../controllers/corp_session_controller.dart';
import '../../l10n/app_localizations.dart';
import '../../widgets/corp_shell.dart';

class CorpHomePage extends StatefulWidget {
  const CorpHomePage({super.key});

  @override
  State<CorpHomePage> createState() => _CorpHomePageState();
}

class _CorpHomePageState extends State<CorpHomePage> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final session = Get.find<CorpSessionController>();
      if (!session.isAuthenticated) {
        Get.offAllNamed('/corp/login');
        return;
      }
      Get.find<CorpController>().refreshProfile();
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final ctrl = Get.find<CorpController>();
    final session = Get.find<CorpSessionController>();
    final theme = Theme.of(context);

    return CorpShell(
      title: l10n.corpHomeTitle,
      child: Obx(() {
        final p = ctrl.profile ?? session.profile.value;
        if (ctrl.loading.value && p == null) {
          return const Center(child: CircularProgressIndicator());
        }
        if (p == null) {
          return Center(child: Text(ctrl.error.value ?? l10n.corpNeedLogin));
        }
        return ListView(
          padding: const EdgeInsets.all(24),
          children: [
            Card(
              child: Padding(
                padding: const EdgeInsets.all(20),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      p.companyName,
                      style: theme.textTheme.headlineSmall?.copyWith(
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text('${l10n.shopColStatus}: ${p.status}'),
                    if (p.contactEmail != null) Text(p.contactEmail!),
                    Text('${l10n.corpMasterCode}: ${p.masterCode}'),
                    if (session.lastApiSecret.value != null) ...[
                      const SizedBox(height: 8),
                      Text(
                        l10n.corpSecretOnce(session.lastApiSecret.value!),
                        style: TextStyle(color: theme.colorScheme.primary),
                      ),
                    ],
                  ],
                ),
              ),
            ),
            if (p.isPending) ...[
              const SizedBox(height: 16),
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Text(l10n.corpPendingApproval),
                ),
              ),
            ],
            if (p.isRejected) ...[
              const SizedBox(height: 16),
              Card(
                color: theme.colorScheme.errorContainer,
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Text(p.rejectReason ?? l10n.corpRejected),
                ),
              ),
            ],
            if (p.isActive) ...[
              const SizedBox(height: 24),
              Align(
                alignment: Alignment.centerLeft,
                child: FilledButton.icon(
                  onPressed: () => Get.toNamed('/corp/games/new'),
                  icon: const Icon(Icons.videogame_asset),
                  label: Text(l10n.corpSubmitGame),
                ),
              ),
            ],
            const SizedBox(height: 24),
            Text(l10n.corpGames, style: theme.textTheme.titleMedium),
            const SizedBox(height: 8),
            if (ctrl.games.isEmpty)
              Text(l10n.corpNoGames)
            else
              ...ctrl.games.map(
                (g) => Card(
                  child: ListTile(
                    title: Text('${g.gameName} (${g.gameCode})'),
                    subtitle: Text(g.status),
                  ),
                ),
              ),
            if (ctrl.error.value != null)
              Padding(
                padding: const EdgeInsets.only(top: 12),
                child: Text(
                  ctrl.error.value!,
                  style: TextStyle(color: theme.colorScheme.error),
                ),
              ),
          ],
        );
      }),
    );
  }
}
