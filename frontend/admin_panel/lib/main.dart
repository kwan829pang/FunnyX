import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:toastification/toastification.dart';

import 'app.dart';
import 'bindings/app_binding.dart';
import 'env/env.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  final config = AppConfig(appName: Env.appName, apiBaseUrl: Env.apiBaseUrl);
  AppBinding(config: config).dependencies();
  runApp(const ToastificationWrapper(child: AdminApp()));
}
