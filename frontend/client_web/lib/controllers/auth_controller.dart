import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import '../services/oauth_flow.dart';

class AuthController extends GetxController {
  AuthController({this.homeRoute = '/home'});

  final String homeRoute;

  final username = ''.obs;
  final password = ''.obs;
  final busy = false.obs;

  AuthDataProvider get _auth => Get.find<AuthDataProvider>();
  SessionController get session => Get.find<SessionController>();
  OauthFlowService get oauth => Get.find<OauthFlowService>();

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
      Get.offAllNamed(homeRoute);
    } catch (e) {
      _toastError(e);
    } finally {
      busy.value = false;
    }
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
      _toastError(e);
    } finally {
      busy.value = false;
    }
  }

  /// Path B: browser leaves Client Web → Partner OAuth via Client Center / STS.
  Future<void> continueWithPartner() async {
    if (busy.value) {
      return;
    }
    busy.value = true;
    try {
      await oauth.startPartnerOauth();
    } catch (e) {
      _toastError(e);
      busy.value = false;
    }
  }

  void applyOauthSession(SessionToken token) {
    session.setToken(token.token);
    Get.offAllNamed(homeRoute);
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
    Get.offAllNamed('/login');
  }

  void _toastError(Object e) {
    toastification.show(
      title: Text('$e'),
      type: ToastificationType.error,
      autoCloseDuration: const Duration(seconds: 3),
    );
  }
}
