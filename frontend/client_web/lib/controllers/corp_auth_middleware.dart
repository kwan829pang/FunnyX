import 'package:flutter/widgets.dart';
import 'package:get/get.dart';

import 'corp_session_controller.dart';

/// Guards corporate portal routes (`/corp/home`, `/corp/games/*`).
///
/// Public corp auth: `/corp/login`, `/corp/register`.
/// End-user Session routes use [AuthMiddleware] separately.
class CorpAuthMiddleware extends GetMiddleware {
  CorpAuthMiddleware({
    this.loginRoute = '/corp/login',
    this.homeRoute = '/corp/home',
  });

  final String loginRoute;
  final String homeRoute;

  bool _isPublic(String? route) {
    if (route == null) return false;
    return route == loginRoute ||
        route == '/corp/register' ||
        route.startsWith('/corp/register');
  }

  @override
  RouteSettings? redirect(String? route) {
    if (!Get.isRegistered<CorpSessionController>()) {
      return RouteSettings(name: loginRoute);
    }
    final session = Get.find<CorpSessionController>();
    final public = _isPublic(route);

    if (!session.isAuthenticated && !public) {
      return RouteSettings(name: loginRoute);
    }
    if (session.isAuthenticated && public) {
      return RouteSettings(name: homeRoute);
    }
    return null;
  }
}
