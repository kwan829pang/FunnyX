import 'package:flutter/material.dart';

/// Horizontal + vertical scroll host for wide [DataTable]s (avoids RIGHT OVERFLOW).
class ScrollableDataTable extends StatelessWidget {
  const ScrollableDataTable({super.key, required this.table});

  final DataTable table;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return Scrollbar(
          thumbVisibility: true,
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: ConstrainedBox(
              constraints: BoxConstraints(minWidth: constraints.maxWidth),
              child: SingleChildScrollView(child: table),
            ),
          ),
        );
      },
    );
  }
}
