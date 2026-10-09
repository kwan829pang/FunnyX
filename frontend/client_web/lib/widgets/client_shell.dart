import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../controllers/auth_controller.dart';
import '../controllers/cart_controller.dart';
import '../l10n/app_localizations.dart';

class ClientShell extends StatelessWidget {
  const ClientShell({super.key, required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final location = Get.currentRoute;
    final auth = Get.find<AuthController>();
    final cart = Get.find<CartController>();
    return ResponsiveScope(
      builder: (context, size) {
        final mobile = size == AppLayoutSize.mobile;
        final index = location.startsWith('/shop') ? 1 : 0;
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
                  onDestinationSelected: (i) {
                    Get.toNamed(i == 0 ? '/home' : '/shop');
                  },
                  labelType: NavigationRailLabelType.all,
                  destinations: [
                    NavigationRailDestination(
                      icon: const Icon(Icons.home_outlined),
                      selectedIcon: const Icon(Icons.home),
                      label: Text(l10n.home),
                    ),
                    NavigationRailDestination(
                      icon: const Icon(Icons.shopping_bag_outlined),
                      selectedIcon: const Icon(Icons.shopping_bag),
                      label: Text(l10n.shop),
                    ),
                  ],
                ),
              Expanded(child: child),
            ],
          ),
          bottomNavigationBar: mobile
              ? NavigationBar(
                  selectedIndex: index,
                  onDestinationSelected: (i) {
                    Get.toNamed(i == 0 ? '/home' : '/shop');
                  },
                  destinations: [
                    NavigationDestination(
                      icon: const Icon(Icons.home),
                      label: l10n.home,
                    ),
                    NavigationDestination(
                      icon: const Icon(Icons.shopping_bag),
                      label: l10n.shop,
                    ),
                  ],
                )
              : null,
        );
      },
    );
  }
}
