import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

import 'client_colors.dart';

/// FunnyX Client Web themes: shop (user) vs corporate portal.
///
/// User theme follows NexaraCart `nexara_cart` light e‑commerce styling
/// (black accent, soft grey surfaces, rounded controls).
abstract final class ClientTheme {
  static bool isCorpRoute(String? route) {
    final r = route ?? '';
    return r == '/corp' || r.startsWith('/corp/');
  }

  static ThemeData forRoute(String? route) =>
      isCorpRoute(route) ? corp() : user();

  /// Light shop theme for normal end users.
  static ThemeData user() {
    final scheme = ColorScheme.fromSeed(
      seedColor: UserColors.darkAccent,
      brightness: Brightness.light,
      primary: UserColors.darkAccent,
      secondary: UserColors.lightAccent,
      surface: UserColors.surface,
    ).copyWith(
      surfaceContainerLowest: UserColors.softBg,
      surfaceContainerLow: UserColors.lightGrey,
      outlineVariant: UserColors.darkGrey.withValues(alpha: 0.45),
    );

    final base = ThemeData(
      useMaterial3: true,
      brightness: Brightness.light,
      colorScheme: scheme,
      scaffoldBackgroundColor: UserColors.softBg,
      visualDensity: VisualDensity.standard,
    );

    final textTheme = GoogleFonts.nunitoTextTheme(base.textTheme).apply(
      bodyColor: UserColors.darkAccent,
      displayColor: UserColors.darkAccent,
    );

    return base.copyWith(
      textTheme: textTheme.copyWith(
        headlineMedium: textTheme.headlineMedium?.copyWith(
          fontWeight: FontWeight.w700,
        ),
        headlineSmall: textTheme.headlineSmall?.copyWith(
          fontWeight: FontWeight.w600,
          color: UserColors.darkGrey,
        ),
        titleMedium: textTheme.titleMedium?.copyWith(
          fontWeight: FontWeight.w600,
        ),
      ),
      appBarTheme: AppBarTheme(
        backgroundColor: Colors.transparent,
        elevation: 0,
        scrolledUnderElevation: 0,
        centerTitle: true,
        foregroundColor: UserColors.darkAccent,
        titleTextStyle: textTheme.titleLarge?.copyWith(
          fontWeight: FontWeight.w700,
          color: UserColors.darkAccent,
        ),
      ),
      cardTheme: CardThemeData(
        color: UserColors.surface,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(UserColors.radius),
          side: BorderSide(color: UserColors.lightGrey),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: UserColors.surface,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(UserColors.radius),
          borderSide: const BorderSide(color: UserColors.lightGrey),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(UserColors.radius),
          borderSide: const BorderSide(color: UserColors.lightGrey),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(UserColors.radius),
          borderSide: const BorderSide(color: UserColors.darkAccent, width: 1.4),
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          backgroundColor: UserColors.darkAccent,
          foregroundColor: Colors.white,
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(UserColors.radius),
          ),
        ),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: UserColors.darkAccent,
          foregroundColor: Colors.white,
          padding: const EdgeInsets.all(12),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(UserColors.radius),
          ),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: UserColors.darkAccent,
          side: const BorderSide(color: UserColors.darkAccent),
          padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 12),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(UserColors.radius),
          ),
        ),
      ),
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(foregroundColor: UserColors.darkAccent),
      ),
      iconTheme: const IconThemeData(color: UserColors.darkGrey),
      navigationBarTheme: NavigationBarThemeData(
        backgroundColor: UserColors.surface,
        indicatorColor: UserColors.lightGrey,
        labelTextStyle: WidgetStatePropertyAll(
          textTheme.labelMedium?.copyWith(fontWeight: FontWeight.w600),
        ),
      ),
      navigationRailTheme: NavigationRailThemeData(
        backgroundColor: UserColors.surface,
        indicatorColor: UserColors.lightGrey,
        selectedIconTheme: const IconThemeData(color: UserColors.darkAccent),
        unselectedIconTheme: const IconThemeData(color: UserColors.darkGrey),
        selectedLabelTextStyle: textTheme.labelMedium?.copyWith(
          color: UserColors.darkAccent,
          fontWeight: FontWeight.w700,
        ),
        unselectedLabelTextStyle: textTheme.labelMedium?.copyWith(
          color: UserColors.darkGrey,
        ),
      ),
      chipTheme: ChipThemeData(
        backgroundColor: UserColors.lightGrey,
        labelStyle: textTheme.labelSmall,
        side: BorderSide.none,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
        ),
      ),
      dividerTheme: const DividerThemeData(color: UserColors.lightGrey),
    );
  }

  /// Corporate portal theme — slate professional layout.
  static ThemeData corp() {
    final scheme = ColorScheme.fromSeed(
      seedColor: CorpColors.primary,
      brightness: Brightness.light,
      primary: CorpColors.primary,
      secondary: CorpColors.accent,
      surface: CorpColors.surface,
    );

    final base = ThemeData(
      useMaterial3: true,
      brightness: Brightness.light,
      colorScheme: scheme,
      scaffoldBackgroundColor: CorpColors.softBg,
      visualDensity: VisualDensity.standard,
    );

    final textTheme = GoogleFonts.sourceSans3TextTheme(base.textTheme).apply(
      bodyColor: CorpColors.primary,
      displayColor: CorpColors.primary,
    );

    return base.copyWith(
      textTheme: textTheme,
      appBarTheme: AppBarTheme(
        backgroundColor: CorpColors.surface,
        elevation: 0,
        scrolledUnderElevation: 0.5,
        foregroundColor: CorpColors.primary,
        titleTextStyle: textTheme.titleLarge?.copyWith(
          fontWeight: FontWeight.w700,
          color: CorpColors.primary,
        ),
      ),
      cardTheme: CardThemeData(
        color: CorpColors.surface,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(CorpColors.radius),
          side: BorderSide(color: CorpColors.muted.withValues(alpha: 0.25)),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: CorpColors.surface,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(CorpColors.radius),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(CorpColors.radius),
          borderSide: BorderSide(color: CorpColors.muted.withValues(alpha: 0.35)),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(CorpColors.radius),
          borderSide: const BorderSide(color: CorpColors.accent, width: 1.4),
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          backgroundColor: CorpColors.accent,
          foregroundColor: Colors.white,
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(CorpColors.radius),
          ),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: CorpColors.primary,
          side: const BorderSide(color: CorpColors.primary),
          padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 12),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(CorpColors.radius),
          ),
        ),
      ),
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(foregroundColor: CorpColors.accent),
      ),
      drawerTheme: const DrawerThemeData(backgroundColor: CorpColors.sidebar),
      navigationRailTheme: const NavigationRailThemeData(
        backgroundColor: CorpColors.sidebar,
        indicatorColor: Color(0x332196F3),
        selectedIconTheme: IconThemeData(color: Colors.white),
        unselectedIconTheme: IconThemeData(color: CorpColors.onSidebar),
        selectedLabelTextStyle: TextStyle(
          color: Colors.white,
          fontWeight: FontWeight.w700,
        ),
        unselectedLabelTextStyle: TextStyle(color: CorpColors.onSidebar),
      ),
    );
  }
}
