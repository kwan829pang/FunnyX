import 'package:flutter/material.dart';

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
