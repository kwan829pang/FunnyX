import 'package:flutter/material.dart';

import '../theme/admin_theme.dart';

/// Dark content card with title / subtitle / optional actions (smart-admin style).
class PageFrame extends StatelessWidget {
  const PageFrame({
    super.key,
    required this.title,
    this.subtitle,
    this.actions,
    required this.child,
  });

  final String title;
  final String? subtitle;
  final List<Widget>? actions;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.all(AdminColors.padding),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(title, style: theme.textTheme.headlineSmall),
                    if (subtitle != null) ...[
                      const SizedBox(height: 8),
                      Text(
                        subtitle!,
                        style: theme.textTheme.bodyMedium?.copyWith(
                          color: Colors.white70,
                        ),
                      ),
                    ],
                  ],
                ),
              ),
              ...?actions,
            ],
          ),
          const SizedBox(height: AdminColors.padding),
          Expanded(
            child: Container(
              width: double.infinity,
              padding: const EdgeInsets.all(AdminColors.padding),
              decoration: BoxDecoration(
                color: AdminColors.secondary,
                borderRadius: BorderRadius.circular(AdminColors.radius),
                border: Border.all(color: Colors.white10),
              ),
              child: child,
            ),
          ),
        ],
      ),
    );
  }
}
