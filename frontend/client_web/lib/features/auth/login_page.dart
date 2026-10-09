import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../../controllers/auth_controller.dart';
import '../../l10n/app_localizations.dart';
import 'oauth_compare_panel.dart';

class LoginPage extends GetView<AuthController> {
  const LoginPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return UnauthGate(
      child: Scaffold(
        body: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 440),
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(24),
              child: Column(
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
                      onPressed:
                          controller.busy.value ? null : controller.login,
                      child: Text(l10n.login),
                    ),
                  ),
                  const SizedBox(height: 8),
                  Obx(
                    () => OutlinedButton.icon(
                      onPressed: controller.busy.value
                          ? null
                          : controller.continueWithPartner,
                      icon: const Icon(Icons.sports_esports_outlined),
                      label: Text(l10n.oauthContinuePartner),
                    ),
                  ),
                  TextButton(
                    onPressed: () => Get.toNamed('/register'),
                    child: Text(l10n.noAccount),
                  ),
                  const SizedBox(height: 24),
                  const Divider(),
                  const SizedBox(height: 16),
                  const OauthComparePanel(showPartnerButton: false),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
