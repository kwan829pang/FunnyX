import 'package:flutter/material.dart';

/// Marker for routes that do not require a session token (login / register / OAuth).
class UnauthGate extends StatelessWidget {
  const UnauthGate({super.key, required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) => child;
}
