import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../controllers/auth_controller.dart';
import '../controllers/cart_controller.dart';
import '../l10n/app_localizations.dart';

class ClientShell extends StatelessWidget {
  const ClientShell({super.key, required this.child});

  final Widget child;

  int _navIndex(String location) {
    if (location.startsWith('/wallet')) return 1;
    if (location.startsWith('/trade')) return 2;
    if (location.startsWith('/shop')) return 3;
    if (location.startsWith('/marketplace')) return 4;
    return 0;
  }

  String _routeForIndex(int i) {
    return switch (i) {
      1 => '/wallet',
      2 => '/trade',
      3 => '/shop',
      4 => '/marketplace',
      _ => '/home',
    };
  }

  // Corp wizards use dedicated routes (/corp/*) outside this shell.

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final location = Get.currentRoute;
    final auth = Get.find<AuthController>();
    final cart = Get.find<CartController>();
    final index = _navIndex(location);
    return ResponsiveScope(
      builder: (context, size) {
        final mobile = size == AppLayoutSize.mobile;
        return Scaffold(
          appBar: mobile
              ? null
              : AppBar(
                  title: Text(l10n.appTitle),
                  actions: [
                    Obx(
                      () => Badge(
                        isLabelVisible: cart.itemCount > 0,
                        label: Text('${cart.itemCount}'),
                        child: IconButton(
                          tooltip: l10n.shopCart,
                          onPressed: () => Get.toNamed('/shop/cart'),
                          icon: const Icon(Icons.shopping_cart_outlined),
                        ),
                      ),
                    ),
                    TextButton(
                      onPressed: () => auth.logout(),
                      child: Text(l10n.logout),
                    ),
                  ],
                ),
          body: Row(
            children: [
              if (!mobile)
                NavigationRail(
                  selectedIndex: index,
                  onDestinationSelected: (i) => Get.toNamed(_routeForIndex(i)),
                  labelType: NavigationRailLabelType.all,
                  destinations: [
                    NavigationRailDestination(
                      icon: const Icon(Icons.home_outlined),
                      selectedIcon: const Icon(Icons.home),
                      label: Text(l10n.home),
                    ),
                    NavigationRailDestination(
                      icon: const Icon(Icons.account_balance_wallet_outlined),
                      selectedIcon: const Icon(Icons.account_balance_wallet),
                      label: Text(l10n.wallet),
                    ),
                    NavigationRailDestination(
                      icon: const Icon(Icons.show_chart_outlined),
                      selectedIcon: const Icon(Icons.show_chart),
                      label: Text(l10n.trade),
                    ),
                    NavigationRailDestination(
                      icon: const Icon(Icons.shopping_bag_outlined),
                      selectedIcon: const Icon(Icons.shopping_bag),
                      label: Text(l10n.shop),
                    ),
                    NavigationRailDestination(
                      icon: const Icon(Icons.storefront_outlined),
                      selectedIcon: const Icon(Icons.storefront),
                      label: Text(l10n.marketplace),
                    ),
                  ],
                ),
              Expanded(child: child),
            ],
          ),
          bottomNavigationBar: mobile
              ? NavigationBar(
                  selectedIndex: index,
                  onDestinationSelected: (i) => Get.toNamed(_routeForIndex(i)),
                  destinations: [
                    NavigationDestination(
                      icon: const Icon(Icons.home),
                      label: l10n.home,
                    ),
                    NavigationDestination(
                      icon: const Icon(Icons.account_balance_wallet),
                      label: l10n.wallet,
                    ),
                    NavigationDestination(
                      icon: const Icon(Icons.show_chart),
                      label: l10n.trade,
                    ),
                    NavigationDestination(
                      icon: const Icon(Icons.shopping_bag),
                      label: l10n.shop,
                    ),
                    NavigationDestination(
                      icon: const Icon(Icons.storefront),
                      label: l10n.marketplace,
                    ),
                  ],
                )
              : null,
        );
      },
    );
  }
}
