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
import '../services/gateway_socket_service.dart';
import '../services/oauth_flow.dart';

class AppBinding extends Bindings {
  AppBinding({required this.config});

  final AppConfig config;

  @override
  void dependencies() {
    Get.put<AppConfig>(config, permanent: true);
    Get.put<ClientThemeController>(ClientThemeController(), permanent: true);
    Get.put<SessionController>(SessionController(), permanent: true);
    Get.put<ApiClient>(
      ApiClient(
        baseUrl: config.apiBaseUrl,
        tokenStore: Get.find<SessionController>(),
      ),
      permanent: true,
    );
    Get.put<AuthDataProvider>(
      ClientCenterAuthProvider(Get.find<ApiClient>()),
      permanent: true,
    );
    Get.put<ShopDataProvider>(
      ClientCenterShopProvider(Get.find<ApiClient>()),
      permanent: true,
    );
    Get.put<WalletDataProvider>(
      ClientCenterWalletProvider(Get.find<ApiClient>()),
      permanent: true,
    );
    Get.put<ExchangeDataProvider>(
      ClientCenterExchangeProvider(Get.find<ApiClient>()),
      permanent: true,
    );
    Get.put<MarketplaceDataProvider>(
      ClientCenterMarketplaceProvider(Get.find<ApiClient>()),
      permanent: true,
    );
    Get.put<CorpSessionController>(CorpSessionController(), permanent: true);
    Get.put<ApiClient>(
      ApiClient(
        baseUrl: config.apiBaseUrl,
        tokenStore: Get.find<CorpSessionController>(),
        publicPathPrefixes: const [
          '/v1/corp/register',
          '/v1/corp/login',
          '/health',
        ],
      ),
      tag: 'corp',
      permanent: true,
    );
    Get.put<CorpDataProvider>(
      ClientCenterCorpProvider(Get.find<ApiClient>(tag: 'corp')),
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
