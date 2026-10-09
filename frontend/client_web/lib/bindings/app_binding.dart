import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../controllers/auth_controller.dart';
import '../controllers/cart_controller.dart';
import '../controllers/shop_catalog_controller.dart';
import '../controllers/shop_checkout_controller.dart';
import '../controllers/shop_orders_controller.dart';
import '../services/oauth_flow.dart';

class AppBinding extends Bindings {
  AppBinding({required this.config});

  final AppConfig config;

  @override
  void dependencies() {
    Get.put<AppConfig>(config, permanent: true);
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
    Get.put<OauthFlowService>(OauthFlowService(config), permanent: true);
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
  }
}
