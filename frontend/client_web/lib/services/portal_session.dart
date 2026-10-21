import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../controllers/corp_session_controller.dart';
import 'gateway_socket_service.dart';

/// Keeps end-user Session and Corp portal sessions isolated.
///
/// Logging into one portal clears the other so Bearer tokens never cross
/// Session APIs (`sts_…`) vs Corp APIs (`corp_…`).
abstract final class PortalSession {
  static void clearEndUser() {
    if (Get.isRegistered<SessionController>()) {
      Get.find<SessionController>().clear();
    }
    if (Get.isRegistered<GatewaySocketService>()) {
      Get.find<GatewaySocketService>().disconnect();
    }
  }

  static void clearCorp() {
    if (Get.isRegistered<CorpSessionController>()) {
      Get.find<CorpSessionController>().clear();
    }
  }

  /// Enter normal-user Session portal — drop any corp Bearer.
  static void enterEndUser(String accessToken) {
    clearCorp();
    Get.find<SessionController>().setToken(accessToken);
  }

  /// Enter corp portal — drop any end-user Session Bearer.
  static void enterCorp(CorpAuthResult result) {
    clearEndUser();
    Get.find<CorpSessionController>().setAuth(result);
  }
}
