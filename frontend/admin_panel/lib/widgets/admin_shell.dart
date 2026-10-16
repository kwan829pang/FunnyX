import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../controllers/auth_controller.dart';
import '../controllers/setup_middleware.dart';
import '../l10n/app_localizations.dart';
import '../theme/admin_theme.dart';

class _NavItem {
  const _NavItem({
    required this.route,
    required this.label,
    required this.icon,
    required this.selectedIcon,
  });

  final String route;
  final String label;
  final IconData icon;
  final IconData selectedIcon;
}

class AdminShell extends StatefulWidget {
  const AdminShell({super.key, required this.child});

  final Widget child;

  @override
  State<AdminShell> createState() => _AdminShellState();
}

class _AdminShellState extends State<AdminShell> {
  final _scaffoldKey = GlobalKey<ScaffoldState>();

  /// Setup nav only while first-time init is incomplete.
  static bool get _showSetup => SetupMiddleware.cachedInitialized != true;

  List<_NavItem> _items(AppLocalizations l10n) {
    final items = <_NavItem>[
      _NavItem(
        route: '/dashboard',
        label: l10n.dashboard,
        icon: Icons.dashboard_outlined,
        selectedIcon: Icons.dashboard,
      ),
      _NavItem(
        route: '/packages',
        label: l10n.packages,
        icon: Icons.inventory_2_outlined,
        selectedIcon: Icons.inventory_2,
      ),
      _NavItem(
        route: '/corp-products',
        label: l10n.corpProducts,
        icon: Icons.store_outlined,
        selectedIcon: Icons.store,
      ),
      _NavItem(
        route: '/markets',
        label: l10n.markets,
        icon: Icons.storefront_outlined,
        selectedIcon: Icons.storefront,
      ),
      const _NavItem(
        route: '/corp-onboard',
        label: 'Onboard',
        icon: Icons.business_outlined,
        selectedIcon: Icons.business,
      ),
      const _NavItem(
        route: '/webhook-auth',
        label: 'Webhook auth',
        icon: Icons.vpn_key_outlined,
        selectedIcon: Icons.vpn_key,
      ),
      const _NavItem(
        route: '/fees',
        label: 'Fees & license',
        icon: Icons.payments_outlined,
        selectedIcon: Icons.payments,
      ),
    ];
    if (_showSetup) {
      items.add(
        _NavItem(
          route: '/setup',
          label: l10n.setup,
          icon: Icons.tune_outlined,
          selectedIcon: Icons.tune,
        ),
      );
    }
    return items;
  }

  String _pageTitle(String location, AppLocalizations l10n) {
    if (location.startsWith('/packages')) return l10n.packages;
    if (location.startsWith('/corp-products')) return l10n.corpProducts;
    if (location.startsWith('/markets')) return l10n.markets;
    if (location.startsWith('/corp-onboard')) return 'Onboard';
    if (location.startsWith('/webhook-auth')) return 'Webhook auth';
    if (location.startsWith('/fees')) return 'Fees & license';
    if (location.startsWith('/setup')) return l10n.setup;
    return l10n.dashboard;
  }

  Future<void> _openPalette(BuildContext context) async {
    final l10n = AppLocalizations.of(context);
    await showDialog<void>(
      context: context,
      builder: (ctx) {
        return AlertDialog(
          backgroundColor: AdminColors.secondary,
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
              } else if (q.contains('onboard') || q.contains('compan')) {
                Get.toNamed('/corp-onboard');
              } else if (q.contains('webhook') ||
                  q.contains('post_auth') ||
                  q.contains('post-auth')) {
                Get.toNamed('/webhook-auth');
              } else if (q.contains('fee') ||
                  q.contains('license') ||
                  q.contains('c6') ||
                  q.contains('c9')) {
                Get.toNamed('/fees');
              } else if (_showSetup && q.contains('setup')) {
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
    final items = _items(l10n);

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
            final desktop = size == AppLayoutSize.desktop;
            return Scaffold(
              key: _scaffoldKey,
              backgroundColor: AdminColors.bg,
              drawer: desktop
                  ? null
                  : Drawer(
                      backgroundColor: AdminColors.secondary,
                      child: _SideMenu(
                        location: location,
                        items: items,
                        onNavigate: (route) {
                          Navigator.of(context).pop();
                          Get.toNamed(route);
                        },
                      ),
                    ),
              body: SafeArea(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    if (desktop)
                      Expanded(
                        child: _SideMenu(
                          location: location,
                          items: items,
                          onNavigate: Get.toNamed,
                        ),
                      ),
                    Expanded(
                      flex: 5,
                      child: Column(
                        children: [
                          _ShellHeader(
                            title: _pageTitle(location, l10n),
                            showMenu: !desktop,
                            onMenu: () =>
                                _scaffoldKey.currentState?.openDrawer(),
                            onSearch: () => _openPalette(context),
                            username: auth.username.value.isEmpty
                                ? 'Admin'
                                : auth.username.value,
                            onLogout: auth.logout,
                            searchLabel: l10n.commandPalette,
                            logoutLabel: l10n.logout,
                          ),
                          Expanded(child: widget.child),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            );
          },
        ),
      ),
    );
  }
}

class _SideMenu extends StatelessWidget {
  const _SideMenu({
    required this.location,
    required this.items,
    required this.onNavigate,
  });

  final String location;
  final List<_NavItem> items;
  final void Function(String route) onNavigate;

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: AdminColors.secondary,
      child: ListView(
        children: [
          DrawerHeader(
            margin: EdgeInsets.zero,
            padding: const EdgeInsets.all(AdminColors.padding),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Container(
                  width: 48,
                  height: 48,
                  decoration: BoxDecoration(
                    color: AdminColors.green.withValues(alpha: 0.2),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: const Icon(
                    Icons.hub_outlined,
                    color: AdminColors.green,
                    size: 28,
                  ),
                ),
                const SizedBox(height: AdminColors.padding),
                Text(
                  'FunnyX Admin',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        color: Colors.white,
                        fontWeight: FontWeight.w600,
                      ),
                ),
              ],
            ),
          ),
          for (final item in items)
            ListTile(
              leading: Icon(
                location.startsWith(item.route)
                    ? item.selectedIcon
                    : item.icon,
                color: location.startsWith(item.route)
                    ? AdminColors.green
                    : Colors.white54,
                size: 20,
              ),
              title: Text(
                item.label,
                style: TextStyle(
                  color: location.startsWith(item.route)
                      ? Colors.white
                      : Colors.white54,
                  fontWeight: location.startsWith(item.route)
                      ? FontWeight.w600
                      : FontWeight.w400,
                ),
              ),
              selected: location.startsWith(item.route),
              selectedTileColor: AdminColors.green.withValues(alpha: 0.12),
              horizontalTitleGap: 8,
              onTap: () => onNavigate(item.route),
            ),
        ],
      ),
    );
  }
}

class _ShellHeader extends StatelessWidget {
  const _ShellHeader({
    required this.title,
    required this.showMenu,
    required this.onMenu,
    required this.onSearch,
    required this.username,
    required this.onLogout,
    required this.searchLabel,
    required this.logoutLabel,
  });

  final String title;
  final bool showMenu;
  final VoidCallback onMenu;
  final VoidCallback onSearch;
  final String username;
  final VoidCallback onLogout;
  final String searchLabel;
  final String logoutLabel;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        AdminColors.padding,
        AdminColors.padding,
        AdminColors.padding,
        0,
      ),
      child: Row(
        children: [
          if (showMenu)
            IconButton(
              onPressed: onMenu,
              icon: const Icon(Icons.menu),
            ),
          if (!showMenu)
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Hello, $username',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
                const SizedBox(height: 4),
                Text(
                  title,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Colors.white60,
                      ),
                ),
              ],
            ),
          if (showMenu)
            Expanded(
              child: Text(
                title,
                style: Theme.of(context).textTheme.titleMedium,
              ),
            )
          else
            const Spacer(flex: 2),
          Expanded(
            child: TextField(
              readOnly: true,
              onTap: onSearch,
              decoration: InputDecoration(
                hintText: searchLabel,
                suffixIcon: Padding(
                  padding: const EdgeInsets.all(6),
                  child: Material(
                    color: AdminColors.green,
                    borderRadius: BorderRadius.circular(10),
                    child: const Icon(Icons.search, color: Colors.white, size: 20),
                  ),
                ),
              ),
            ),
          ),
          const SizedBox(width: AdminColors.padding),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: AdminColors.secondary,
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: Colors.white10),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                CircleAvatar(
                  radius: 14,
                  backgroundColor: AdminColors.green.withValues(alpha: 0.3),
                  child: Text(
                    username.isNotEmpty ? username[0].toUpperCase() : 'A',
                    style: const TextStyle(fontSize: 12, color: Colors.white),
                  ),
                ),
                if (!showMenu) ...[
                  const SizedBox(width: 8),
                  Text(username),
                ],
                PopupMenuButton<String>(
                  tooltip: logoutLabel,
                  onSelected: (v) {
                    if (v == 'logout') onLogout();
                  },
                  itemBuilder: (_) => [
                    PopupMenuItem(value: 'logout', child: Text(logoutLabel)),
                  ],
                  child: const Icon(Icons.keyboard_arrow_down, size: 20),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
