import 'package:flutter/widgets.dart';
import 'package:get/get.dart';

import 'session_controller.dart';

/// Redirects unauthenticated users to [loginRoute].
class AuthMiddleware extends GetMiddleware {
  AuthMiddleware({
    this.loginRoute = '/login',
    this.homeRoute = '/home',
  });

  final String loginRoute;
  final String homeRoute;

  @override
  RouteSettings? redirect(String? route) {
    if (!Get.isRegistered<SessionController>()) {
      return RouteSettings(name: loginRoute);
    }
    final session = Get.find<SessionController>();
    final public = route == loginRoute ||
        route == '/register' ||
        (route != null && route.startsWith('/oauth'));
    if (!session.isAuthenticated && !public) {
      return RouteSettings(name: loginRoute);
    }
    // Allow OAuth callback even if somehow flagged; do not bounce authenticated
    // users away from completing token handoff.
    if (session.isAuthenticated &&
        public &&
        route != null &&
        !route.startsWith('/oauth')) {
      return RouteSettings(name: homeRoute);
    }
    return null;
  }
}
