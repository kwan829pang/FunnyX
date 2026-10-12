import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/corp_controller.dart';
import '../../l10n/app_localizations.dart';

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
    return Scaffold(
      appBar: AppBar(title: Text(l10n.corpLoginTitle)),
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 420),
          child: ListView(
            padding: const EdgeInsets.all(24),
            children: [
              TextField(
                controller: _login,
                decoration: InputDecoration(labelText: l10n.corpLoginField),
              ),
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
                  child: Text(err, style: TextStyle(color: Theme.of(context).colorScheme.error)),
                );
              }),
              TextButton(
                onPressed: () => Get.toNamed('/corp/register'),
                child: Text(l10n.corpNoAccount),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
