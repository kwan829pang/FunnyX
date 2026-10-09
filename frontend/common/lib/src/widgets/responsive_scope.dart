import 'package:flutter/material.dart';

import '../theme/breakpoints.dart';

enum AppLayoutSize { mobile, tablet, desktop }

/// Rebuilds children when the window crosses layout breakpoints.
class ResponsiveScope extends StatelessWidget {
  const ResponsiveScope({super.key, required this.builder});

  final Widget Function(BuildContext context, AppLayoutSize size) builder;

  static AppLayoutSize of(BuildContext context) {
    final width = MediaQuery.sizeOf(context).width;
    if (Breakpoints.isMobile(width)) {
      return AppLayoutSize.mobile;
    }
    if (Breakpoints.isTablet(width)) {
      return AppLayoutSize.tablet;
    }
    return AppLayoutSize.desktop;
  }

  @override
  Widget build(BuildContext context) {
    return builder(context, of(context));
  }
}
