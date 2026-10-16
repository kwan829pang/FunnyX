import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import 'l10n/app_localizations.dart';
import 'routes/app_pages.dart';
import 'theme/admin_theme.dart';

class AdminApp extends StatelessWidget {
  const AdminApp({super.key});

  @override
  Widget build(BuildContext context) {
    final config = Get.find<AppConfig>();
    return GetMaterialApp(
      title: config.appName,
      debugShowCheckedModeBanner: false,
      theme: AdminTheme.dark(),
      darkTheme: AdminTheme.dark(),
      themeMode: ThemeMode.dark,
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
    );
  }
}
