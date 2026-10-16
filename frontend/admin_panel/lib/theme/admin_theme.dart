import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

/// Visual tokens inspired by deniscolak/smart-admin-dashboard.
class AdminColors {
  static const Color primaryBlue = Color(0xFF2697FF);
  static const Color secondary = Color(0xFF292929);
  static const Color bg = Color(0xFF212121);
  static const Color darkGreen = Color(0xFF2C614F);
  static const Color green = Color(0xFF6BAB58);
  static const Color lightPanel = Color(0xFFEDEEF2);

  static const double padding = 16;
  static const double radius = 15;
}

class AdminTheme {
  static ThemeData dark() {
    final base = ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      scaffoldBackgroundColor: AdminColors.bg,
      canvasColor: AdminColors.secondary,
      colorScheme: const ColorScheme.dark(
        primary: AdminColors.green,
        secondary: AdminColors.primaryBlue,
        surface: AdminColors.secondary,
        onPrimary: Colors.white,
        onSecondary: Colors.white,
        onSurface: Colors.white,
      ),
    );

    final textTheme = GoogleFonts.openSansTextTheme(base.textTheme).apply(
      bodyColor: Colors.white,
      displayColor: Colors.white,
    );

    return base.copyWith(
      textTheme: textTheme,
      primaryColor: AdminColors.green,
      appBarTheme: const AppBarTheme(
        backgroundColor: AdminColors.bg,
        elevation: 0,
        foregroundColor: Colors.white,
      ),
      cardTheme: CardThemeData(
        color: AdminColors.secondary,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(AdminColors.radius),
          side: const BorderSide(color: Colors.white10),
        ),
      ),
      drawerTheme: const DrawerThemeData(
        backgroundColor: AdminColors.secondary,
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: AdminColors.secondary,
        hintStyle: const TextStyle(color: Colors.white54),
        labelStyle: const TextStyle(color: Colors.white70),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(10),
          borderSide: BorderSide.none,
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(10),
          borderSide: const BorderSide(color: Colors.white10),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(10),
          borderSide: const BorderSide(color: AdminColors.green),
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          backgroundColor: AdminColors.green,
          foregroundColor: Colors.white,
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(10),
          ),
        ),
      ),
      dataTableTheme: const DataTableThemeData(
        headingTextStyle: TextStyle(
          color: Colors.white70,
          fontWeight: FontWeight.w600,
        ),
        dataTextStyle: TextStyle(color: Colors.white),
      ),
      dividerTheme: const DividerThemeData(color: Colors.white12),
      listTileTheme: const ListTileThemeData(
        iconColor: Colors.white54,
        textColor: Colors.white54,
        selectedColor: Colors.white,
        selectedTileColor: Color(0x22269BFF),
      ),
    );
  }
}
