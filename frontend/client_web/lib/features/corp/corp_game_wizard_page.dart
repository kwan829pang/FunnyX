import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import '../../controllers/corp_controller.dart';
import '../../l10n/app_localizations.dart';
import '../../widgets/corp_shell.dart';

class CorpGameWizardPage extends StatefulWidget {
  const CorpGameWizardPage({super.key});

  @override
  State<CorpGameWizardPage> createState() => _CorpGameWizardPageState();
}

class _CorpGameWizardPageState extends State<CorpGameWizardPage> {
  final _name = TextEditingController();
  final _code = TextEditingController();
  final _partnerCode = TextEditingController();
  final _partnerGameId = TextEditingController();
  final _description = TextEditingController();
  PlatformFile? _gov;
  PlatformFile? _material;

  @override
  void dispose() {
    _name.dispose();
    _code.dispose();
    _partnerCode.dispose();
    _partnerGameId.dispose();
    _description.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final ctrl = Get.find<CorpController>();
    return CorpShell(
      title: l10n.corpSubmitGame,
      showBack: true,
      child: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 560),
          child: ListView(
            padding: const EdgeInsets.all(24),
            children: [
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(20),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Text(l10n.corpGameWizardHint),
                      const SizedBox(height: 16),
                      TextField(
                        controller: _name,
                        decoration:
                            InputDecoration(labelText: l10n.corpGameName),
                      ),
                      const SizedBox(height: 12),
                      TextField(
                        controller: _code,
                        decoration:
                            InputDecoration(labelText: l10n.corpGameCode),
                      ),
                      const SizedBox(height: 12),
                      TextField(
                        controller: _partnerCode,
                        decoration:
                            InputDecoration(labelText: l10n.corpPartnerCode),
                      ),
                      const SizedBox(height: 12),
                      TextField(
                        controller: _partnerGameId,
                        decoration:
                            InputDecoration(labelText: l10n.corpPartnerGameId),
                      ),
                      const SizedBox(height: 12),
                      TextField(
                        controller: _description,
                        maxLines: 3,
                        decoration: InputDecoration(
                          labelText: l10n.corpGameDescription,
                        ),
                      ),
                      const SizedBox(height: 12),
                      OutlinedButton.icon(
                        onPressed: () => _pick(true),
                        icon: const Icon(Icons.gavel),
                        label: Text(
                          _gov == null
                              ? l10n.corpUploadGovDoc
                              : '${l10n.corpGovSelected}: ${_gov!.name}',
                        ),
                      ),
                      const SizedBox(height: 8),
                      OutlinedButton.icon(
                        onPressed: () => _pick(false),
                        icon: const Icon(Icons.description),
                        label: Text(
                          _material == null
                              ? l10n.corpUploadMaterial
                              : '${l10n.corpMaterialSelected}: ${_material!.name}',
                        ),
                      ),
                      const SizedBox(height: 24),
                      Obx(
                        () => FilledButton(
                          onPressed: ctrl.loading.value ? null : _submit,
                          child: Text(l10n.corpSubmitGame),
                        ),
                      ),
                      Obx(() {
                        final err = ctrl.error.value;
                        if (err == null) return const SizedBox.shrink();
                        return Padding(
                          padding: const EdgeInsets.only(top: 12),
                          child: Text(
                            err,
                            style: TextStyle(
                              color: Theme.of(context).colorScheme.error,
                            ),
                          ),
                        );
                      }),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _pick(bool government) async {
    final result = await FilePicker.platform.pickFiles(withData: true);
    if (result != null && result.files.isNotEmpty) {
      setState(() {
        if (government) {
          _gov = result.files.first;
        } else {
          _material = result.files.first;
        }
      });
    }
  }

  Future<void> _submit() async {
    final l10n = AppLocalizations.of(context);
    final ctrl = Get.find<CorpController>();
    if ((_gov?.bytes == null) && (_material?.bytes == null)) {
      toastification.show(
        context: context,
        type: ToastificationType.warning,
        title: Text(l10n.corpGameDocRequired),
        autoCloseDuration: const Duration(seconds: 3),
      );
      return;
    }
    final game = await ctrl.submitGame(
      gameName: _name.text.trim(),
      gameCode: _code.text.trim(),
      partnerCode: _partnerCode.text.trim(),
      partnerGameId: _partnerGameId.text.trim(),
      description: _description.text.trim(),
      governmentBytes: _gov?.bytes,
      governmentFileName: _gov?.name,
      materialBytes: _material?.bytes,
      materialFileName: _material?.name,
    );
    if (!mounted) return;
    if (game != null) {
      toastification.show(
        context: context,
        type: ToastificationType.success,
        title: Text(l10n.corpGameSubmitted),
        autoCloseDuration: const Duration(seconds: 3),
      );
      Get.offNamed('/corp/home');
    }
  }
}
