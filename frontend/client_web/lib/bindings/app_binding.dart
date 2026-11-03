import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../controllers/auth_controller.dart';
import '../controllers/cart_controller.dart';
import '../controllers/client_theme_controller.dart';
import '../controllers/corp_controller.dart';
import '../controllers/corp_session_controller.dart';
import '../controllers/marketplace_controller.dart';
import '../controllers/shop_catalog_controller.dart';
import '../controllers/shop_checkout_controller.dart';
import '../controllers/shop_orders_controller.dart';
import '../controllers/trade_controller.dart';
import '../controllers/wallet_controller.dart';
import '../services/auth_failure_handler.dart';
import '../services/gateway_socket_service.dart';
import '../services/oauth_flow.dart';

class AppBinding extends Bindings {
  AppBinding({required this.config});

  final AppConfig config;

  @override
  void dependencies() {
    Get.put<AppConfig>(config, permanent: true);
    Get.put<ClientThemeController>(ClientThemeController(), permanent: true);
    final session = SessionController(
      prefsAccessKey: 'fx_client_access',
      prefsRefreshKey: 'fx_client_refresh',
    );
    Get.put<SessionController>(session, permanent: true);

    // End-user Session APIs (wallet/profile/shop/…) — must use SessionController token.
    final sessionApi = ApiClient(
      baseUrl: config.apiBaseUrl,
      tokenStore: session,
      onUnauthorized: AuthFailureHandler.sessionUnauthorized,
    );
    Get.put<ApiClient>(sessionApi, permanent: true);
    Get.put<ApiClient>(sessionApi, tag: 'session', permanent: true);

    Get.put<AuthDataProvider>(
      ClientCenterAuthProvider(sessionApi),
      permanent: true,
    );
    Get.put<ShopDataProvider>(
      ClientCenterShopProvider(sessionApi),
      permanent: true,
    );
    Get.put<WalletDataProvider>(
      ClientCenterWalletProvider(sessionApi),
      permanent: true,
    );
    Get.put<ExchangeDataProvider>(
      ClientCenterExchangeProvider(sessionApi),
      permanent: true,
    );
    Get.put<MarketplaceDataProvider>(
      ClientCenterMarketplaceProvider(sessionApi),
      permanent: true,
    );

    final corpSession = CorpSessionController();
    Get.put<CorpSessionController>(corpSession, permanent: true);
    final corpApi = ApiClient(
      baseUrl: config.apiBaseUrl,
      tokenStore: corpSession,
      onUnauthorized: AuthFailureHandler.corpUnauthorized,
      publicPathPrefixes: const [
        '/v1/corp/register',
        '/v1/corp/login',
        '/health',
      ],
    );
    Get.put<ApiClient>(corpApi, tag: 'corp', permanent: true);
    Get.put<CorpDataProvider>(
      ClientCenterCorpProvider(corpApi),
      permanent: true,
    );
    Get.put<CorpController>(
      CorpController(
        Get.find<CorpDataProvider>(),
        Get.find<CorpSessionController>(),
      ),
      permanent: true,
    );
    Get.put<OauthFlowService>(OauthFlowService(config), permanent: true);
    Get.put<GatewaySocketService>(
      GatewaySocketService(config),
      permanent: true,
    );
    Get.put<AuthController>(
      AuthController(homeRoute: '/home'),
      permanent: true,
    );
    Get.put<CartController>(CartController(), permanent: true);
    Get.put<ShopCatalogController>(
      ShopCatalogController(Get.find<ShopDataProvider>()),
      permanent: true,
    );
    Get.put<ShopCheckoutController>(
      ShopCheckoutController(Get.find<ShopDataProvider>()),
      permanent: true,
    );
    Get.put<ShopOrdersController>(
      ShopOrdersController(Get.find<ShopDataProvider>()),
      permanent: true,
    );
    Get.put<MarketplaceController>(
      MarketplaceController(
        Get.find<MarketplaceDataProvider>(),
        Get.find<ShopDataProvider>(),
      ),
      permanent: true,
    );
    Get.put<WalletController>(WalletController(), permanent: true);
    Get.put<TradeController>(TradeController(), permanent: true);
  }
}
