import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../features/auth/login_page.dart';
import '../features/auth/oauth_callback_page.dart';
import '../features/auth/register_page.dart';
import '../features/corp/corp_game_wizard_page.dart';
import '../features/corp/corp_home_page.dart';
import '../features/corp/corp_login_page.dart';
import '../features/corp/corp_register_page.dart';
import '../features/home/home_page.dart';
import '../features/marketplace/marketplace_deal_page.dart';
import '../features/marketplace/marketplace_page.dart';
import '../features/marketplace/marketplace_place_page.dart';
import '../features/shop/cart_page.dart';
import '../features/shop/checkout_page.dart';
import '../features/shop/order_detail_page.dart';
import '../features/shop/orders_page.dart';
import '../features/shop/payment_page.dart';
import '../features/shop/shop_page.dart';
import '../widgets/client_shell.dart';

abstract class AppPages {
  static const initial = '/login';

  static final authGuard = AuthMiddleware(
    loginRoute: '/login',
    homeRoute: '/home',
  );

  static final routes = <GetPage<dynamic>>[
    GetPage(
      name: '/login',
      page: () => const LoginPage(),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/register',
      page: () => const RegisterPage(),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/oauth/callback',
      page: () => const OauthCallbackPage(),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/home',
      page: () => const ClientShell(child: HomePage()),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/shop',
      page: () => const ClientShell(child: ShopPage()),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/shop/cart',
      page: () => const ClientShell(child: CartPage()),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/shop/checkout',
      page: () => const ClientShell(child: CheckoutPage()),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/shop/payment',
      page: () => const ClientShell(child: PaymentPage()),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/shop/orders',
      page: () => const ClientShell(child: OrdersPage()),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/shop/orders/:id',
      page: () {
        final id = int.tryParse(Get.parameters['id'] ?? '') ?? 0;
        return ClientShell(child: OrderDetailPage(orderId: id));
      },
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/marketplace',
      page: () => const ClientShell(child: MarketplacePage()),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/marketplace/new',
      page: () => const ClientShell(child: MarketplacePlacePage()),
      middlewares: [authGuard],
    ),
    GetPage(
      name: '/marketplace/deals/:id',
      page: () {
        final id = int.tryParse(Get.parameters['id'] ?? '') ?? 0;
        return ClientShell(child: MarketplaceDealPage(dealId: id));
      },
      middlewares: [authGuard],
    ),
    GetPage(name: '/corp/register', page: () => const CorpRegisterPage()),
    GetPage(name: '/corp/login', page: () => const CorpLoginPage()),
    GetPage(name: '/corp/home', page: () => const CorpHomePage()),
    GetPage(name: '/corp/games/new', page: () => const CorpGameWizardPage()),
  ];
}
