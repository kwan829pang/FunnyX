import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import '../../controllers/corp_controller.dart';
import '../../l10n/app_localizations.dart';

class CorpRegisterPage extends StatefulWidget {
  const CorpRegisterPage({super.key});

  @override
  State<CorpRegisterPage> createState() => _CorpRegisterPageState();
}

class _CorpRegisterPageState extends State<CorpRegisterPage> {
  final _company = TextEditingController();
  final _contact = TextEditingController();
  final _email = TextEditingController();
  final _password = TextEditingController();
  PlatformFile? _br;

  @override
  void dispose() {
    _company.dispose();
    _contact.dispose();
    _email.dispose();
    _password.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final ctrl = Get.find<CorpController>();
    return Scaffold(
      appBar: AppBar(title: Text(l10n.corpRegisterTitle)),
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 520),
          child: ListView(
            padding: const EdgeInsets.all(24),
            children: [
              Text(l10n.corpRegisterHint),
              const SizedBox(height: 16),
              TextField(
                controller: _company,
                decoration: InputDecoration(labelText: l10n.corpCompanyName),
              ),
              TextField(
                controller: _contact,
                decoration: InputDecoration(labelText: l10n.corpContactName),
              ),
              TextField(
                controller: _email,
                decoration: InputDecoration(labelText: l10n.corpContactEmail),
              ),
              TextField(
                controller: _password,
                obscureText: true,
                decoration: InputDecoration(labelText: l10n.password),
              ),
              const SizedBox(height: 12),
              OutlinedButton.icon(
                onPressed: _pickBr,
                icon: const Icon(Icons.upload_file),
                label: Text(
                  _br == null ? l10n.corpUploadBr : '${l10n.corpBrSelected}: ${_br!.name}',
                ),
              ),
              const SizedBox(height: 24),
              Obx(
                () => FilledButton(
                  onPressed: ctrl.loading.value ? null : _submit,
                  child: Text(l10n.corpSubmitRegistration),
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
                onPressed: () => Get.toNamed('/corp/login'),
                child: Text(l10n.corpHaveAccount),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _pickBr() async {
    final result = await FilePicker.platform.pickFiles(withData: true);
    if (result != null && result.files.isNotEmpty) {
      setState(() => _br = result.files.first);
    }
  }

  Future<void> _submit() async {
    final l10n = AppLocalizations.of(context);
    final ctrl = Get.find<CorpController>();
    final br = _br;
    if (br?.bytes == null) {
      toastification.show(
        context: context,
        type: ToastificationType.warning,
        title: Text(l10n.corpBrRequired),
        autoCloseDuration: const Duration(seconds: 3),
      );
      return;
    }
    final ok = await ctrl.register(
      companyName: _company.text.trim(),
      contactName: _contact.text.trim(),
      contactEmail: _email.text.trim(),
      password: _password.text,
      brBytes: br!.bytes!,
      brFileName: br.name,
    );
    if (!mounted) return;
    if (ok) {
      toastification.show(
        context: context,
        type: ToastificationType.success,
        title: Text(l10n.corpRegisterSubmitted),
        autoCloseDuration: const Duration(seconds: 3),
      );
      Get.offAllNamed('/corp/home');
    }
  }
}
