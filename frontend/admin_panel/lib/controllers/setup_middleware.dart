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
    if (route == loginRoute) {
      return null;
    }
    if (!Get.isRegistered<SessionController>()) {
      return null;
    }
    if (!Get.find<SessionController>().isAuthenticated) {
      return null;
    }
    // First-time setup finished → hide /setup (nav + deep link).
    if (route == setupRoute && cachedInitialized == true) {
      return const RouteSettings(name: '/dashboard');
    }
    if (route == setupRoute) {
      return null;
    }
    if (cachedInitialized == false) {
      return RouteSettings(name: setupRoute);
    }
    return null;
  }
}
