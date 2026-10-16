import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../../controllers/auth_controller.dart';
import '../../l10n/app_localizations.dart';
import '../../widgets/auth_layout.dart';
import 'oauth_compare_panel.dart';

class LoginPage extends GetView<AuthController> {
  const LoginPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final theme = Theme.of(context);
    return UnauthGate(
      child: AuthLayout(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              l10n.appTitle,
              style: theme.textTheme.headlineMedium?.copyWith(
                fontWeight: FontWeight.w800,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              l10n.loginHint,
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.65),
              ),
            ),
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
            TextButton(
              onPressed: () => Get.toNamed('/corp/login'),
              child: Text(l10n.corpLoginTitle),
            ),
            const SizedBox(height: 16),
            const Divider(),
            const SizedBox(height: 16),
            const OauthComparePanel(showPartnerButton: false),
          ],
        ),
      ),
    );
  }
}
