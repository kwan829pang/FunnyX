import 'package:get/get.dart';

import '../theme/client_theme.dart';

/// Tracks whether the active route is the corporate portal so UI can
/// switch between [ClientTheme.user] and [ClientTheme.corp].
class ClientThemeController extends GetxController {
  final isCorp = false.obs;

  void syncFromRoute([Routing? routing]) {
    final name = routing?.current ?? Get.currentRoute;
    final next = ClientTheme.isCorpRoute(name);
    if (isCorp.value != next) {
      isCorp.value = next;
    }
  }
}
