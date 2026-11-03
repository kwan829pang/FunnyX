import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import 'portal_session.dart';

/// Coalesces 401 / missing-Bearer handling so parallel calls do not spam toasts
/// or fight over navigation.
abstract final class AuthFailureHandler {
  static bool _sessionBusy = false;
  static bool _corpBusy = false;

  static void sessionUnauthorized() {
    if (_sessionBusy) return;
    _sessionBusy = true;
    try {
      PortalSession.clearEndUser();
      final route = Get.currentRoute;
      if (route.startsWith('/corp')) return;
      if (route == '/login' ||
          route == '/register' ||
          route.startsWith('/oauth')) {
        return;
      }
      Get.offAllNamed('/login');
    } finally {
      Future<void>.delayed(const Duration(milliseconds: 400), () {
        _sessionBusy = false;
      });
    }
  }

  static void corpUnauthorized() {
    if (_corpBusy) return;
    _corpBusy = true;
    try {
      PortalSession.clearCorp();
      final route = Get.currentRoute;
      if (!route.startsWith('/corp')) return;
      if (route == '/corp/login' || route == '/corp/register') return;
      Get.offAllNamed('/corp/login');
    } finally {
      Future<void>.delayed(const Duration(milliseconds: 400), () {
        _corpBusy = false;
      });
    }
  }

  /// True when [error] is an auth failure that navigation already handles.
  static bool isAuthFailure(Object error) {
    if (error is ApiError) {
      if (error.isUnauthorized) return true;
      return _looksExpired(error.message);
    }
    return _looksExpired(error.toString());
  }

  static bool _looksExpired(String raw) {
    final text = raw.toLowerCase();
    return text.contains('401') ||
        text.contains('unauthorized') ||
        text.contains('invalid_or_expired') ||
        text.contains('invalid or expired') ||
        text.contains('invalid session') ||
        text.contains('missing session token') ||
        text.contains('missing bearer');
  }
}
