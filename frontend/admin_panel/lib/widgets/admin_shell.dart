import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../controllers/auth_controller.dart';
import '../l10n/app_localizations.dart';

class AdminShell extends StatelessWidget {
  const AdminShell({super.key, required this.child});

  final Widget child;

  int _selectedIndex(String location) {
    if (location.startsWith('/packages')) return 1;
    if (location.startsWith('/corp-products')) return 2;
    if (location.startsWith('/markets')) return 3;
    if (location.startsWith('/setup')) return 4;
    return 0;
  }

  String _routeForIndex(int i) {
    switch (i) {
      case 1:
        return '/packages';
      case 2:
        return '/corp-products';
      case 3:
        return '/markets';
      case 4:
        return '/setup';
      default:
        return '/dashboard';
    }
  }

  Future<void> _openPalette(BuildContext context) async {
    final l10n = AppLocalizations.of(context);
    await showDialog<void>(
      context: context,
      builder: (ctx) {
        return AlertDialog(
          title: Text(l10n.commandPalette),
          content: TextField(
            autofocus: true,
            decoration: InputDecoration(hintText: l10n.commandPalette),
            onSubmitted: (value) {
              Navigator.of(ctx).pop();
              final q = value.toLowerCase();
              if (q.contains('package') || q.contains('plt')) {
                Get.toNamed('/packages');
              } else if (q.contains('corp') || q.contains('product')) {
                Get.toNamed('/corp-products');
              } else if (q.contains('market')) {
                Get.toNamed('/markets');
              } else if (q.contains('setup')) {
                Get.toNamed('/setup');
              } else {
                Get.toNamed('/dashboard');
              }
            },
          ),
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final location = Get.currentRoute;
    final auth = Get.find<AuthController>();
    final selected = _selectedIndex(location);
    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.keyK, control: true):
            () => _openPalette(context),
        const SingleActivator(LogicalKeyboardKey.keyK, meta: true):
            () => _openPalette(context),
      },
      child: Focus(
        autofocus: true,
        child: ResponsiveScope(
          builder: (context, size) {
            final mobile = size == AppLayoutSize.mobile;
            return Scaffold(
              appBar: AppBar(
                title: Text(l10n.appTitle),
                actions: [
                  IconButton(
                    tooltip: l10n.commandPalette,
                    onPressed: () => _openPalette(context),
                    icon: const Icon(Icons.search),
                  ),
                  TextButton(
                    onPressed: () => auth.logout(),
                    child: Text(l10n.logout),
                  ),
                ],
              ),
              drawer: mobile
                  ? Drawer(child: _NavList(location: location, l10n: l10n))
                  : null,
              body: Row(
                children: [
                  if (!mobile)
                    NavigationRail(
                      selectedIndex: selected,
                      onDestinationSelected: (i) => Get.toNamed(_routeForIndex(i)),
                      labelType: NavigationRailLabelType.all,
                      destinations: [
                        NavigationRailDestination(
                          icon: const Icon(Icons.dashboard_outlined),
                          selectedIcon: const Icon(Icons.dashboard),
                          label: Text(l10n.dashboard),
                        ),
                        NavigationRailDestination(
                          icon: const Icon(Icons.inventory_2_outlined),
                          selectedIcon: const Icon(Icons.inventory_2),
                          label: Text(l10n.packages),
                        ),
                        NavigationRailDestination(
                          icon: const Icon(Icons.store_outlined),
                          selectedIcon: const Icon(Icons.store),
                          label: Text(l10n.corpProducts),
                        ),
                        NavigationRailDestination(
                          icon: const Icon(Icons.storefront_outlined),
                          selectedIcon: const Icon(Icons.storefront),
                          label: Text(l10n.markets),
                        ),
                        NavigationRailDestination(
                          icon: const Icon(Icons.tune_outlined),
                          selectedIcon: const Icon(Icons.tune),
                          label: Text(l10n.setup),
                        ),
                      ],
                    ),
                  Expanded(child: child),
                ],
              ),
            );
          },
        ),
      ),
    );
  }
}

class _NavList extends StatelessWidget {
  const _NavList({required this.location, required this.l10n});

  final String location;
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    return ListView(
      children: [
        ListTile(
          leading: const Icon(Icons.dashboard),
          title: Text(l10n.dashboard),
          selected: location.startsWith('/dashboard'),
          onTap: () => Get.toNamed('/dashboard'),
        ),
        ListTile(
          leading: const Icon(Icons.inventory_2),
          title: Text(l10n.packages),
          selected: location.startsWith('/packages'),
          onTap: () => Get.toNamed('/packages'),
        ),
        ListTile(
          leading: const Icon(Icons.store),
          title: Text(l10n.corpProducts),
          selected: location.startsWith('/corp-products'),
          onTap: () => Get.toNamed('/corp-products'),
        ),
        ListTile(
          leading: const Icon(Icons.storefront),
          title: Text(l10n.markets),
          selected: location.startsWith('/markets'),
          onTap: () => Get.toNamed('/markets'),
        ),
        ListTile(
          leading: const Icon(Icons.tune),
          title: Text(l10n.setup),
          selected: location.startsWith('/setup'),
          onTap: () => Get.toNamed('/setup'),
        ),
      ],
    );
  }
}
