import 'package:flutter/material.dart';

import '../theme/client_colors.dart';
import '../theme/client_theme.dart';

/// Centered auth card for user (shop) or corp portals.
class AuthLayout extends StatelessWidget {
  const AuthLayout({
    super.key,
    required this.child,
    this.maxWidth = 440,
    this.corp = false,
  });

  final Widget child;
  final double maxWidth;
  final bool corp;

  @override
  Widget build(BuildContext context) {
    final theme = corp ? ClientTheme.corp() : ClientTheme.user();
    final bg = corp
        ? const LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: [Color(0xFF0B2A40), Color(0xFF1F6B8A), Color(0xFFEEF3F7)],
            stops: [0.0, 0.45, 1.0],
          )
        : const LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            colors: [UserColors.lightGrey, UserColors.softBg, Colors.white],
          );

    return Theme(
      data: theme,
      child: Scaffold(
        body: DecoratedBox(
          decoration: BoxDecoration(gradient: bg),
          child: SafeArea(
            child: Center(
              child: SingleChildScrollView(
                padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 32),
                child: ConstrainedBox(
                  constraints: BoxConstraints(maxWidth: maxWidth),
                  child: Material(
                    color: corp ? CorpColors.surface : UserColors.surface,
                    elevation: 0,
                    borderRadius: BorderRadius.circular(
                      corp ? CorpColors.radius + 4 : UserColors.radius + 4,
                    ),
                    child: Container(
                      decoration: BoxDecoration(
                        borderRadius: BorderRadius.circular(
                          corp ? CorpColors.radius + 4 : UserColors.radius + 4,
                        ),
                        border: Border.all(
                          color: corp
                              ? CorpColors.muted.withValues(alpha: 0.2)
                              : UserColors.lightGrey,
                        ),
                        boxShadow: [
                          BoxShadow(
                            color: Colors.black.withValues(alpha: 0.06),
                            blurRadius: 24,
                            offset: const Offset(0, 8),
                          ),
                        ],
                      ),
                      padding: const EdgeInsets.all(28),
                      child: child,
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
