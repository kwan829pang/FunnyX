import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../controllers/corp_controller.dart';
import '../l10n/app_localizations.dart';
import '../theme/client_colors.dart';

/// Corporate portal chrome — dark sidebar + content (distinct from shop shell).
class CorpShell extends StatelessWidget {
  const CorpShell({
    super.key,
    required this.child,
    this.title,
    this.showBack = false,
  });

  final Widget child;
  final String? title;
  final bool showBack;

  int _navIndex(String location) {
    if (location.startsWith('/corp/games')) return 1;
    return 0;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final location = Get.currentRoute;
    final index = _navIndex(location);
    final ctrl = Get.find<CorpController>();

    return ResponsiveScope(
      builder: (context, size) {
        final mobile = size == AppLayoutSize.mobile;
        final destinations = [
          _CorpDest(
            icon: Icons.dashboard_outlined,
            selectedIcon: Icons.dashboard,
            label: l10n.corpHomeTitle,
            route: '/corp/home',
          ),
          _CorpDest(
            icon: Icons.videogame_asset_outlined,
            selectedIcon: Icons.videogame_asset,
            label: l10n.corpSubmitGame,
            route: '/corp/games/new',
          ),
        ];

        void go(int i) {
          final route = destinations[i].route;
          if (Get.currentRoute != route) Get.toNamed(route);
        }

        final sidebar = ColoredBox(
          color: CorpColors.sidebar,
          child: SizedBox(
            width: 240,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Padding(
                  padding: const EdgeInsets.fromLTRB(20, 28, 20, 12),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        l10n.appTitle,
                        style: Theme.of(context).textTheme.titleLarge?.copyWith(
                              color: CorpColors.onSidebar,
                              fontWeight: FontWeight.w800,
                            ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        l10n.corpHomeTitle,
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                              color:
                                  CorpColors.onSidebar.withValues(alpha: 0.75),
                            ),
                      ),
                    ],
                  ),
                ),
                const Divider(color: Colors.white12, height: 1),
                const SizedBox(height: 8),
                for (var i = 0; i < destinations.length; i++)
                  _CorpNavTile(
                    selected: index == i,
                    icon: index == i
                        ? destinations[i].selectedIcon
                        : destinations[i].icon,
                    label: destinations[i].label,
                    onTap: () => go(i),
                  ),
                const Spacer(),
                Padding(
                  padding: const EdgeInsets.all(16),
                  child: TextButton.icon(
                    style: TextButton.styleFrom(
                      foregroundColor: CorpColors.onSidebar,
                      alignment: Alignment.centerLeft,
                    ),
                    onPressed: () {
                      ctrl.logout();
                      Get.offAllNamed('/corp/login');
                    },
                    icon: const Icon(Icons.logout),
                    label: Text(l10n.logout),
                  ),
                ),
              ],
            ),
          ),
        );

        return Scaffold(
          backgroundColor: CorpColors.softBg,
          appBar: mobile
              ? AppBar(
                  title: Text(title ?? l10n.corpHomeTitle),
                  leading: showBack
                      ? IconButton(
                          icon: const Icon(Icons.arrow_back),
                          onPressed: () => Get.back(),
                        )
                      : null,
                  actions: [
                    TextButton(
                      onPressed: () {
                        ctrl.logout();
                        Get.offAllNamed('/corp/login');
                      },
                      child: Text(l10n.logout),
                    ),
                  ],
                )
              : AppBar(
                  title: Text(title ?? l10n.corpHomeTitle),
                  leading: showBack
                      ? IconButton(
                          icon: const Icon(Icons.arrow_back),
                          onPressed: () => Get.back(),
                        )
                      : const SizedBox.shrink(),
                  leadingWidth: showBack ? null : 0,
                ),
          drawer: mobile
              ? Drawer(
                  backgroundColor: CorpColors.sidebar,
                  child: SafeArea(child: sidebar),
                )
              : null,
          body: Row(
            children: [
              if (!mobile) sidebar,
              Expanded(
                child: ColoredBox(
                  color: CorpColors.softBg,
                  child: child,
                ),
              ),
            ],
          ),
          bottomNavigationBar: mobile
              ? NavigationBar(
                  selectedIndex: index.clamp(0, destinations.length - 1),
                  onDestinationSelected: go,
                  destinations: [
                    for (final d in destinations)
                      NavigationDestination(
                        icon: Icon(d.icon),
                        selectedIcon: Icon(d.selectedIcon),
                        label: d.label,
                      ),
                  ],
                )
              : null,
        );
      },
    );
  }
}

class _CorpDest {
  const _CorpDest({
    required this.icon,
    required this.selectedIcon,
    required this.label,
    required this.route,
  });

  final IconData icon;
  final IconData selectedIcon;
  final String label;
  final String route;
}

class _CorpNavTile extends StatelessWidget {
  const _CorpNavTile({
    required this.selected,
    required this.icon,
    required this.label,
    required this.onTap,
  });

  final bool selected;
  final IconData icon;
  final String label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 2),
      child: ListTile(
        selected: selected,
        selectedTileColor: Colors.white12,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
        leading: Icon(
          icon,
          color: selected ? Colors.white : CorpColors.onSidebar,
        ),
        title: Text(
          label,
          style: TextStyle(
            color: selected ? Colors.white : CorpColors.onSidebar,
            fontWeight: selected ? FontWeight.w700 : FontWeight.w500,
          ),
        ),
        onTap: onTap,
      ),
    );
  }
}
