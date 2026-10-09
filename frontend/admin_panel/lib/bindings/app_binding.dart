import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../controllers/auth_controller.dart';
import '../controllers/corp_products_controller.dart';
import '../controllers/packages_controller.dart';
import '../controllers/setup_controller.dart';

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
      AdminApiAuthProvider(Get.find<ApiClient>()),
      permanent: true,
    );
    Get.put<AdminSystemProvider>(
      AdminSystemProvider(Get.find<ApiClient>()),
      permanent: true,
    );
    Get.put<AuthController>(AuthController(), permanent: true);
    Get.lazyPut<SetupController>(() => SetupController(), fenix: true);
    Get.lazyPut<PackagesController>(() => PackagesController(), fenix: true);
    Get.lazyPut<CorpProductsController>(
      () => CorpProductsController(),
      fenix: true,
    );
  }
}
