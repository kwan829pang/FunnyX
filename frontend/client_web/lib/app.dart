import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import 'controllers/client_theme_controller.dart';
import 'l10n/app_localizations.dart';
import 'routes/app_pages.dart';
import 'theme/client_theme.dart';

class ClientApp extends StatelessWidget {
  const ClientApp({super.key});

  @override
  Widget build(BuildContext context) {
    final config = Get.find<AppConfig>();
    final themeCtrl = Get.find<ClientThemeController>();
    themeCtrl.syncFromRoute();

    return GetMaterialApp(
      title: config.appName,
      theme: ClientTheme.user(),
      themeMode: ThemeMode.light,
      initialRoute: AppPages.initial,
      getPages: AppPages.routes,
      locale: const Locale('en'),
      supportedLocales: AppLocalizations.supportedLocales,
      localizationsDelegates: const [
        AppLocalizations.delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      routingCallback: themeCtrl.syncFromRoute,
      builder: (context, child) {
        return Obx(() {
          final data =
              themeCtrl.isCorp.value ? ClientTheme.corp() : ClientTheme.user();
          return Theme(
            data: data,
            child: child ?? const SizedBox.shrink(),
          );
        });
      },
    );
  }
}
