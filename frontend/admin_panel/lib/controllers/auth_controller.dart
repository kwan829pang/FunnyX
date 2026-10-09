import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import 'setup_middleware.dart';

class AuthController extends GetxController {
  AuthController({
    this.homeRoute = '/dashboard',
    this.setupRoute = '/setup',
  });

  final String homeRoute;
  final String setupRoute;

  final username = ''.obs;
  final password = ''.obs;
  final busy = false.obs;

  AuthDataProvider get _auth => Get.find<AuthDataProvider>();
  SessionController get session => Get.find<SessionController>();

  Future<void> login() async {
    if (busy.value) {
      return;
    }
    busy.value = true;
    try {
      final token = await _auth.login(
        username: username.value.trim(),
        password: password.value,
      );
      session.setToken(token.token);
      await _routeAfterAuth();
    } catch (e) {
      toastification.show(
        title: Text('$e'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 3),
      );
    } finally {
      busy.value = false;
    }
  }

  Future<void> _routeAfterAuth() async {
    if (!Get.isRegistered<AdminSystemProvider>()) {
      Get.offAllNamed(homeRoute);
      return;
    }
    try {
      final status = await Get.find<AdminSystemProvider>().fetchSetup();
      SetupMiddleware.markInitialized(status.initialized);
      if (!status.initialized) {
        Get.offAllNamed(setupRoute);
        return;
      }
    } catch (_) {
      SetupMiddleware.markInitialized(null);
    }
    Get.offAllNamed(homeRoute);
  }

  Future<void> register() async {
    if (busy.value) {
      return;
    }
    busy.value = true;
    try {
      final token = await _auth.register(
        username: username.value.trim(),
        password: password.value,
      );
      session.setToken(token.token);
      Get.offAllNamed(homeRoute);
    } catch (e) {
      toastification.show(
        title: Text('$e'),
        type: ToastificationType.error,
        autoCloseDuration: const Duration(seconds: 3),
      );
    } finally {
      busy.value = false;
    }
  }

  Future<void> logout() async {
    final access = session.token;
    try {
      await _auth.logout(accessToken: access);
    } catch (_) {
      // Local clear even if revoke fails.
    }
    session.clear();
    username.value = '';
    password.value = '';
    SetupMiddleware.markInitialized(null);
    Get.offAllNamed('/login');
  }
}
