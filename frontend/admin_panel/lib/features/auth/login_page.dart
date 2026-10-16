import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

import '../../controllers/auth_controller.dart';
import '../../l10n/app_localizations.dart';
import '../../theme/admin_theme.dart';

class LoginPage extends GetView<AuthController> {
  const LoginPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return UnauthGate(
      child: Scaffold(
        backgroundColor: AdminColors.bg,
        body: LayoutBuilder(
          builder: (context, constraints) {
            final wide = constraints.maxWidth >= 900;
            if (!wide) {
              return Center(
                child: SingleChildScrollView(
                  padding: const EdgeInsets.all(24),
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 420),
                    child: _LoginForm(l10n: l10n),
                  ),
                ),
              );
            }
            return Row(
              children: [
                Expanded(
                  child: Container(
                    color: AdminColors.lightPanel,
                    padding: const EdgeInsets.all(48),
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Container(
                          width: 64,
                          height: 64,
                          decoration: BoxDecoration(
                            color: AdminColors.green.withValues(alpha: 0.15),
                            borderRadius: BorderRadius.circular(16),
                          ),
                          child: const Icon(
                            Icons.hub_outlined,
                            size: 36,
                            color: AdminColors.darkGreen,
                          ),
                        ),
                        const SizedBox(height: 24),
                        Text(
                          l10n.appTitle,
                          style: Theme.of(context)
                              .textTheme
                              .headlineLarge
                              ?.copyWith(
                                color: const Color(0xFF1A1A1A),
                                fontWeight: FontWeight.w700,
                              ),
                        ),
                        const SizedBox(height: 12),
                        Text(
                          l10n.loginHint,
                          style: Theme.of(context)
                              .textTheme
                              .titleMedium
                              ?.copyWith(color: Colors.black54),
                        ),
                        const SizedBox(height: 32),
                        Text(
                          'Markets · Corp onboard · Fees · Webhook auth',
                          style: Theme.of(context)
                              .textTheme
                              .bodyMedium
                              ?.copyWith(color: Colors.black45),
                        ),
                      ],
                    ),
                  ),
                ),
                Expanded(
                  child: ColoredBox(
                    color: AdminColors.bg,
                    child: Center(
                      child: ConstrainedBox(
                        constraints: const BoxConstraints(maxWidth: 400),
                        child: Padding(
                          padding: const EdgeInsets.all(42),
                          child: _LoginForm(l10n: l10n),
                        ),
                      ),
                    ),
                  ),
                ),
              ],
            );
          },
        ),
      ),
    );
  }
}

class _LoginForm extends GetView<AuthController> {
  const _LoginForm({required this.l10n});

  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          l10n.login,
          style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                color: Colors.white,
                fontWeight: FontWeight.w600,
              ),
        ),
        const SizedBox(height: 8),
        Text(
          l10n.loginHint,
          style: const TextStyle(color: Colors.white60),
        ),
        const SizedBox(height: 28),
        TextField(
          style: const TextStyle(color: Colors.white),
          decoration: InputDecoration(
            labelText: l10n.username,
            hintText: 'seed_admin',
          ),
          onChanged: (v) => controller.username.value = v,
          onSubmitted: (_) => controller.login(),
        ),
        const SizedBox(height: 12),
        TextField(
          obscureText: true,
          style: const TextStyle(color: Colors.white),
          decoration: InputDecoration(
            labelText: l10n.password,
            hintText: '********',
          ),
          onChanged: (v) => controller.password.value = v,
          onSubmitted: (_) => controller.login(),
        ),
        const SizedBox(height: 28),
        Obx(
          () => FilledButton(
            onPressed: controller.busy.value ? null : controller.login,
            child: controller.busy.value
                ? const SizedBox(
                    height: 18,
                    width: 18,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      color: Colors.white,
                    ),
                  )
                : Text(l10n.login),
          ),
        ),
      ],
    );
  }
}
