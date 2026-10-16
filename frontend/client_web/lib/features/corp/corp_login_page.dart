import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/corp_controller.dart';
import '../../l10n/app_localizations.dart';
import '../../widgets/auth_layout.dart';

class CorpLoginPage extends StatefulWidget {
  const CorpLoginPage({super.key});

  @override
  State<CorpLoginPage> createState() => _CorpLoginPageState();
}

class _CorpLoginPageState extends State<CorpLoginPage> {
  final _login = TextEditingController();
  final _password = TextEditingController();

  @override
  void dispose() {
    _login.dispose();
    _password.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final ctrl = Get.find<CorpController>();
    final theme = Theme.of(context);
    return AuthLayout(
      corp: true,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            l10n.corpLoginTitle,
            style: theme.textTheme.headlineMedium?.copyWith(
              fontWeight: FontWeight.w800,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            l10n.corpHomeTitle,
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.onSurface.withValues(alpha: 0.65),
            ),
          ),
          const SizedBox(height: 24),
          TextField(
            controller: _login,
            decoration: InputDecoration(labelText: l10n.corpLoginField),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _password,
            obscureText: true,
            decoration: InputDecoration(labelText: l10n.password),
          ),
          const SizedBox(height: 24),
          Obx(
            () => FilledButton(
              onPressed: ctrl.loading.value
                  ? null
                  : () async {
                      final ok = await ctrl.login(
                        login: _login.text.trim(),
                        password: _password.text,
                      );
                      if (ok) Get.offAllNamed('/corp/home');
                    },
              child: Text(l10n.login),
            ),
          ),
          Obx(() {
            final err = ctrl.error.value;
            if (err == null) return const SizedBox.shrink();
            return Padding(
              padding: const EdgeInsets.only(top: 12),
              child: Text(
                err,
                style: TextStyle(color: theme.colorScheme.error),
              ),
            );
          }),
          TextButton(
            onPressed: () => Get.toNamed('/corp/register'),
            child: Text(l10n.corpNoAccount),
          ),
          TextButton(
            onPressed: () => Get.offAllNamed('/login'),
            child: Text(l10n.appTitle),
          ),
        ],
      ),
    );
  }
}
