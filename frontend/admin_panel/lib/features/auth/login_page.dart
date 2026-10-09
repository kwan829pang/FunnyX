import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../../controllers/auth_controller.dart';
import '../../l10n/app_localizations.dart';

class LoginPage extends GetView<AuthController> {
  const LoginPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return UnauthGate(
      child: Scaffold(
        body: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 400),
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Text(
                    l10n.appTitle,
                    style: Theme.of(context).textTheme.headlineMedium,
                  ),
                  const SizedBox(height: 8),
                  Text(l10n.loginHint),
                  const SizedBox(height: 24),
                  TextField(
                    decoration: InputDecoration(labelText: l10n.username),
                    onChanged: (v) => controller.username.value = v,
                    onSubmitted: (_) => controller.login(),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    obscureText: true,
                    decoration: InputDecoration(labelText: l10n.password),
                    onChanged: (v) => controller.password.value = v,
                    onSubmitted: (_) => controller.login(),
                  ),
                  const SizedBox(height: 24),
                  Obx(
                    () => FilledButton(
                      onPressed: controller.busy.value ? null : controller.login,
                      child: controller.busy.value
                          ? const SizedBox(
                              height: 18,
                              width: 18,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : Text(l10n.login),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
