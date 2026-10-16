import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/setup_middleware.dart';
import '../../l10n/app_localizations.dart';
import '../../theme/admin_theme.dart';

class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final showSetup = SetupMiddleware.cachedInitialized != true;
    final cards = <_DashCard>[
      _DashCard(
        title: l10n.markets,
        subtitle: 'Approve & activate pairs',
        icon: Icons.storefront_outlined,
        color: AdminColors.primaryBlue,
        route: '/markets',
      ),
      _DashCard(
        title: l10n.packages,
        subtitle: 'PLT shop catalog',
        icon: Icons.inventory_2_outlined,
        color: AdminColors.green,
        route: '/packages',
      ),
      _DashCard(
        title: 'Onboard',
        subtitle: 'Corp & game review',
        icon: Icons.business_outlined,
        color: AdminColors.darkGreen,
        route: '/corp-onboard',
      ),
      _DashCard(
        title: 'Fees',
        subtitle: 'License & platform fees',
        icon: Icons.payments_outlined,
        color: const Color(0xFFE0A800),
        route: '/fees',
      ),
    ];

    return Padding(
      padding: const EdgeInsets.all(AdminColors.padding),
      child: LayoutBuilder(
        builder: (context, constraints) {
          final wide = constraints.maxWidth >= 960;
          return SingleChildScrollView(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text(
                  l10n.dashboard,
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
                const SizedBox(height: 8),
                Text(
                  'Welcome to your FunnyX control plane.',
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: Colors.white70,
                      ),
                ),
                const SizedBox(height: AdminColors.padding),
                Wrap(
                  spacing: AdminColors.padding,
                  runSpacing: AdminColors.padding,
                  children: [
                    for (final c in cards)
                      SizedBox(
                        width: wide
                            ? (constraints.maxWidth -
                                    AdminColors.padding * 3) /
                                4
                            : constraints.maxWidth,
                        child: _MiniInfoCard(card: c),
                      ),
                  ],
                ),
                const SizedBox(height: AdminColors.padding),
                Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Expanded(
                      flex: 5,
                      child: Container(
                        padding: const EdgeInsets.all(AdminColors.padding),
                        decoration: BoxDecoration(
                          color: AdminColors.secondary,
                          borderRadius:
                              BorderRadius.circular(AdminColors.radius),
                          border: Border.all(color: Colors.white10),
                        ),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'Quick links',
                              style: Theme.of(context).textTheme.titleMedium,
                            ),
                            const SizedBox(height: 12),
                            Wrap(
                              spacing: 8,
                              runSpacing: 8,
                              children: [
                                _QuickChip(
                                  label: l10n.corpProducts,
                                  onTap: () => Get.toNamed('/corp-products'),
                                ),
                                _QuickChip(
                                  label: 'Webhook auth',
                                  onTap: () => Get.toNamed('/webhook-auth'),
                                ),
                                if (showSetup)
                                  _QuickChip(
                                    label: l10n.setup,
                                    onTap: () => Get.toNamed('/setup'),
                                  ),
                              ],
                            ),
                          ],
                        ),
                      ),
                    ),
                    if (wide) ...[
                      const SizedBox(width: AdminColors.padding),
                      Expanded(
                        flex: 2,
                        child: Container(
                          padding:
                              const EdgeInsets.all(AdminColors.padding),
                          decoration: BoxDecoration(
                            color: AdminColors.secondary,
                            borderRadius:
                                BorderRadius.circular(AdminColors.radius),
                            border: Border.all(color: Colors.white10),
                          ),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                'Tips',
                                style: Theme.of(context).textTheme.titleMedium,
                              ),
                              const SizedBox(height: 12),
                              Text(
                                '- Demo login: seed_admin / demo\n'
                                '- Markets stay pending until you Approve\n'
                                '- System setup hides after first init',
                                style: Theme.of(context)
                                    .textTheme
                                    .bodySmall
                                    ?.copyWith(color: Colors.white70, height: 1.5),
                              ),
                            ],
                          ),
                        ),
                      ),
                    ],
                  ],
                ),
              ],
            ),
          );
        },
      ),
    );
  }
}

class _DashCard {
  const _DashCard({
    required this.title,
    required this.subtitle,
    required this.icon,
    required this.color,
    required this.route,
  });

  final String title;
  final String subtitle;
  final IconData icon;
  final Color color;
  final String route;
}

class _MiniInfoCard extends StatelessWidget {
  const _MiniInfoCard({required this.card});

  final _DashCard card;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: AdminColors.secondary,
      borderRadius: BorderRadius.circular(AdminColors.radius),
      child: InkWell(
        borderRadius: BorderRadius.circular(AdminColors.radius),
        onTap: () => Get.toNamed(card.route),
        child: Container(
          padding: const EdgeInsets.all(AdminColors.padding),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(AdminColors.radius),
            border: Border.all(color: Colors.white10),
          ),
          child: Row(
            children: [
              Container(
                width: 48,
                height: 48,
                decoration: BoxDecoration(
                  color: card.color.withValues(alpha: 0.2),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Icon(card.icon, color: card.color),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      card.title,
                      style: Theme.of(context).textTheme.titleSmall?.copyWith(
                            fontWeight: FontWeight.w600,
                          ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      card.subtitle,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Colors.white54,
                          ),
                    ),
                  ],
                ),
              ),
              const Icon(Icons.chevron_right, color: Colors.white38),
            ],
          ),
        ),
      ),
    );
  }
}

class _QuickChip extends StatelessWidget {
  const _QuickChip({required this.label, required this.onTap});

  final String label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return ActionChip(
      label: Text(label),
      onPressed: onTap,
      backgroundColor: AdminColors.bg,
      side: const BorderSide(color: Colors.white12),
    );
  }
}
