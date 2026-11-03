import 'package:flutter/material.dart';

/// Shared spacing for cards, dialogs, inputs, and page containers.
abstract final class ClientSpacing {
  static const double xs = 4;
  static const double sm = 8;
  static const double md = 12;
  static const double lg = 16;
  static const double xl = 24;
  static const double xxl = 32;

  /// Gap between stacked form fields (avoids floating-label overlap).
  static const double fieldGap = 16;

  static const EdgeInsets page = EdgeInsets.all(xl);
  static const EdgeInsets cardPadding = EdgeInsets.all(20);
  static const EdgeInsets cardMargin = EdgeInsets.zero;

  static const EdgeInsets inputContent =
      EdgeInsets.symmetric(horizontal: 16, vertical: 16);

  static const EdgeInsets dialogInset =
      EdgeInsets.symmetric(horizontal: 24, vertical: 24);
  static const EdgeInsets dialogTitle =
      EdgeInsets.fromLTRB(24, 24, 24, 12);
  static const EdgeInsets dialogContent =
      EdgeInsets.fromLTRB(24, 8, 24, 8);
  static const EdgeInsets dialogActions =
      EdgeInsets.fromLTRB(24, 12, 24, 20);

  static const double dialogRadius = 20;
}

/// End-user shop palette inspired by
/// [NexaraCart](https://github.com/Mahmud0808/NexaraCart) `nexara_cart`.
abstract final class UserColors {
  static const Color darkAccent = Color(0xFF121212);
  static const Color lightAccent = Color(0xFF9AB7C5);
  static const Color darkGrey = Color(0xFFB0B2B7);
  static const Color lightGrey = Color(0xFFE6E8EC);
  static const Color softBg = Color(0xFFF7F8FA);
  static const Color surface = Colors.white;
  static const double radius = 12;
}

/// Corporate portal palette — cooler slate, distinct from the shop UI.
abstract final class CorpColors {
  static const Color primary = Color(0xFF0F3D5E);
  static const Color accent = Color(0xFF1F6B8A);
  static const Color softBg = Color(0xFFEEF3F7);
  static const Color surface = Colors.white;
  static const Color sidebar = Color(0xFF0B2A40);
  static const Color onSidebar = Color(0xFFE8F1F6);
  static const Color muted = Color(0xFF6B7C8A);
  static const double radius = 10;
}
