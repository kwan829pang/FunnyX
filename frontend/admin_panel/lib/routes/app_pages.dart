import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../controllers/setup_middleware.dart';
import '../features/auth/login_page.dart';
import '../features/corp_onboard/corp_onboard_page.dart';
import '../features/corp_products/corp_products_page.dart';
import '../features/dashboard/dashboard_page.dart';
import '../features/fees/fees_page.dart';
import '../features/markets/markets_page.dart';
import '../features/packages/packages_page.dart';
import '../features/setup/setup_wizard_page.dart';
import '../features/webhook_auth/webhook_auth_page.dart';
import '../widgets/admin_shell.dart';

abstract class AppPages {
  static const initial = '/login';

  static final authGuard = AuthMiddleware(
    loginRoute: '/login',
    homeRoute: '/dashboard',
  );

  static final setupGuard = SetupMiddleware(
    setupRoute: '/setup',
    loginRoute: '/login',
  );

  static final routes = <GetPage<dynamic>>[
    GetPage(
      name: '/login',
      page: () => const LoginPage(),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/setup',
      page: () => const SetupWizardPage(),
      middlewares: [authGuard, setupGuard],
    ),
    GetPage(
      name: '/dashboard',
      page: () => const AdminShell(child: DashboardPage()),
      middlewares: [authGuard, setupGuard],
    ),
    GetPage(
      name: '/packages',
      page: () => const AdminShell(child: PackagesPage()),
      middlewares: [authGuard, setupGuard],
    ),
    GetPage(
      name: '/corp-products',
      page: () => const AdminShell(child: CorpProductsPage()),
      middlewares: [authGuard, setupGuard],
    ),
    GetPage(
      name: '/markets',
      page: () => const AdminShell(child: MarketsPage()),
      middlewares: [authGuard, setupGuard],
    ),
    GetPage(
      name: '/corp-onboard',
      page: () => const AdminShell(child: CorpOnboardPage()),
      middlewares: [authGuard, setupGuard],
    ),
    GetPage(
      name: '/webhook-auth',
      page: () => const AdminShell(child: WebhookAuthPage()),
      middlewares: [authGuard, setupGuard],
    ),
    GetPage(
      name: '/fees',
      page: () => const AdminShell(child: FeesPage()),
      middlewares: [authGuard, setupGuard],
    ),
  ];
}
