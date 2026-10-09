import 'package:flutter/material.dart';

/// Light/dark Material 3 theme aligned with a slate / shadcn-like palette.
class AppTheme {
  static const Color _seed = Color(0xFF0F172A);

  static ThemeData light() {
    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.light,
      colorScheme: ColorScheme.fromSeed(
        seedColor: _seed,
        brightness: Brightness.light,
      ),
      visualDensity: VisualDensity.standard,
    );
  }

  static ThemeData dark() {
    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      colorScheme: ColorScheme.fromSeed(
        seedColor: _seed,
        brightness: Brightness.dark,
      ),
      visualDensity: VisualDensity.standard,
    );
  }
}
