import 'package:flutter/widgets.dart';
import 'package:get/get.dart';

import 'session_controller.dart';

/// Redirects unauthenticated **end users** to [loginRoute].
///
/// Corporate portal (`/corp/*`) is a separate auth domain — this middleware
/// never redirects those routes (they use CorpAuthMiddleware on Client Web).
class AuthMiddleware extends GetMiddleware {
  AuthMiddleware({
    this.loginRoute = '/login',
    this.homeRoute = '/home',
  });

  final String loginRoute;
  final String homeRoute;

  bool _isCorpRoute(String? route) =>
      route == '/corp' || (route != null && route.startsWith('/corp/'));

  bool _isPublic(String? route) {
    if (route == null) return false;
    return route == loginRoute ||
        route == '/register' ||
        route.startsWith('/oauth');
  }

  @override
  RouteSettings? redirect(String? route) {
    // Corp portal has its own session + middleware.
    if (_isCorpRoute(route)) {
      return null;
    }
    if (!Get.isRegistered<SessionController>()) {
      return RouteSettings(name: loginRoute);
    }
    final session = Get.find<SessionController>();
    final public = _isPublic(route);
    if (!session.isAuthenticated && !public) {
      return RouteSettings(name: loginRoute);
    }
    if (session.isAuthenticated &&
        public &&
        route != null &&
        !route.startsWith('/oauth')) {
      return RouteSettings(name: homeRoute);
    }
    return null;
  }
}
