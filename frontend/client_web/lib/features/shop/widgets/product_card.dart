import 'package:flutter/material.dart';

import '../../../theme/client_colors.dart';

class ProductCard extends StatelessWidget {
  const ProductCard({
    super.key,
    required this.title,
    required this.code,
    required this.sellerLabel,
    required this.priceLabel,
    required this.onAdd,
  });

  final String title;
  final String code;
  final String sellerLabel;
  final String priceLabel;
  final VoidCallback onAdd;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Material(
      color: UserColors.surface,
      elevation: 0,
      borderRadius: BorderRadius.circular(UserColors.radius),
      child: InkWell(
        borderRadius: BorderRadius.circular(UserColors.radius),
        onTap: onAdd,
        child: Ink(
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(UserColors.radius),
            border: Border.all(color: UserColors.lightGrey),
          ),
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Chip(
                      label:
                          Text(sellerLabel, style: theme.textTheme.labelSmall),
                      visualDensity: VisualDensity.compact,
                      padding: EdgeInsets.zero,
                    ),
                    const Spacer(),
                    Text(
                      code,
                      style: theme.textTheme.labelMedium?.copyWith(
                        color: UserColors.darkGrey,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                Text(
                  title,
                  style: theme.textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.w700,
                  ),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
                const Spacer(),
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        priceLabel,
                        style: theme.textTheme.titleSmall?.copyWith(
                          fontWeight: FontWeight.w700,
                        ),
                      ),
                    ),
                    FilledButton(
                      onPressed: onAdd,
                      style: FilledButton.styleFrom(
                        padding: const EdgeInsets.all(10),
                        minimumSize: const Size(40, 40),
                      ),
                      child: const Icon(Icons.add_shopping_cart, size: 18),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
