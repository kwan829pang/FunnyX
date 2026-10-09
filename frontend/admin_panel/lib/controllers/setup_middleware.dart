import 'package:flutter/widgets.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

/// Sync gate: when [cachedInitialized] is false, send authenticated users to `/setup`.
class SetupMiddleware extends GetMiddleware {
  SetupMiddleware({
    this.setupRoute = '/setup',
    this.loginRoute = '/login',
  });

  final String setupRoute;
  final String loginRoute;

  /// null = unknown (do not block); false = force setup; true = allow app.
  static bool? cachedInitialized;

  static void markInitialized(bool? value) {
    cachedInitialized = value;
  }

  @override
  RouteSettings? redirect(String? route) {
    if (route == loginRoute || route == setupRoute) {
      return null;
    }
    if (!Get.isRegistered<SessionController>()) {
      return null;
    }
    if (!Get.find<SessionController>().isAuthenticated) {
      return null;
    }
    if (cachedInitialized == false) {
      return RouteSettings(name: setupRoute);
    }
    return null;
  }
}
