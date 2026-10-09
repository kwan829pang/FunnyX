import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../controllers/setup_controller.dart';
import '../../l10n/app_localizations.dart';

/// First-run wizard for required system variables (base fiat + PLT prices).
class SetupWizardPage extends GetView<SetupController> {
  const SetupWizardPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.setupWizardTitle),
      ),
      body: Obx(() {
        if (controller.loading.value) {
          return const Center(child: CircularProgressIndicator());
        }
        return Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 720),
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Text(
                    l10n.setupWizardSubtitle,
                    style: Theme.of(context).textTheme.bodyLarge,
                  ),
                  const SizedBox(height: 16),
                  Obx(
                    () => Stepper(
                      currentStep: controller.step.value,
                      onStepTapped: (i) => controller.step.value = i,
                      controlsBuilder: (context, details) {
                        return const SizedBox.shrink();
                      },
                      steps: [
                        Step(
                          title: Text(l10n.setupStepWelcome),
                          isActive: controller.step.value >= 0,
                          state: controller.step.value > 0
                              ? StepState.complete
                              : StepState.indexed,
                          content: _WelcomeStep(l10n: l10n),
                        ),
                        Step(
                          title: Text(l10n.setupStepBaseFiat),
                          isActive: controller.step.value >= 1,
                          state: (controller.setup.value?.baseFiatReady ?? false)
                              ? StepState.complete
                              : StepState.indexed,
                          content: _BaseFiatStep(l10n: l10n),
                        ),
                        Step(
                          title: Text(l10n.setupStepPackages),
                          isActive: controller.step.value >= 2,
                          state: (controller.setup.value?.packagesReady ?? false)
                              ? StepState.complete
                              : StepState.indexed,
                          content: _PackagesStep(l10n: l10n),
                        ),
                        Step(
                          title: Text(l10n.setupStepConfirm),
                          isActive: controller.step.value >= 3,
                          state: (controller.setup.value?.initialized ?? false)
                              ? StepState.complete
                              : StepState.indexed,
                          content: _ConfirmStep(l10n: l10n),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
        );
      }),
    );
  }
}

class _WelcomeStep extends StatelessWidget {
  const _WelcomeStep({required this.l10n});

  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    final c = Get.find<SetupController>();
    final required = c.setup.value?.required ?? const [];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(l10n.setupWelcomeBody),
        const SizedBox(height: 12),
        for (final item in required)
          ListTile(
            dense: true,
            leading: Icon(
              item.ready ? Icons.check_circle : Icons.radio_button_unchecked,
              color: item.ready ? Colors.green : null,
            ),
            title: Text(item.label),
            subtitle: Text(item.detail),
          ),
        const SizedBox(height: 12),
        FilledButton(
          onPressed: () => c.step.value = 1,
          child: Text(l10n.setupNext),
        ),
      ],
    );
  }
}

class _BaseFiatStep extends StatelessWidget {
  const _BaseFiatStep({required this.l10n});

  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    final c = Get.find<SetupController>();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(l10n.setupBaseFiatHint),
        const SizedBox(height: 12),
        Obx(
          () => SegmentedButton<String>(
            segments: const [
              ButtonSegment(value: 'HKD', label: Text('HKD')),
              ButtonSegment(value: 'USD', label: Text('USD')),
            ],
            selected: {c.baseFiat.value},
            onSelectionChanged: (s) => c.baseFiat.value = s.first,
          ),
        ),
        const SizedBox(height: 16),
        Obx(
          () => FilledButton(
            onPressed: c.busy.value ? null : c.saveBaseCurrency,
            child: c.busy.value
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : Text(l10n.setupSaveContinue),
          ),
        ),
      ],
    );
  }
}

class _PackagesStep extends StatelessWidget {
  const _PackagesStep({required this.l10n});

  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    final c = Get.find<SetupController>();
    final packages = c.setup.value?.packages ?? const [];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(l10n.setupPackagesHint),
        const SizedBox(height: 12),
        for (final p in packages)
          Padding(
            padding: const EdgeInsets.only(bottom: 8),
            child: Row(
              children: [
                Expanded(
                  flex: 2,
                  child: Text('${p.code} (${p.coinAmount.toStringAsFixed(0)} PLT)'),
                ),
                Expanded(
                  child: TextField(
                    controller: c.packagePrices[p.id],
                    keyboardType: const TextInputType.numberWithOptions(decimal: true),
                    decoration: InputDecoration(
                      labelText: l10n.setupFiatPrice,
                      isDense: true,
                    ),
                  ),
                ),
              ],
            ),
          ),
        const SizedBox(height: 8),
        Obx(
          () => FilledButton(
            onPressed: c.busy.value ? null : c.savePackages,
            child: c.busy.value
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : Text(l10n.setupSaveContinue),
          ),
        ),
      ],
    );
  }
}

class _ConfirmStep extends StatelessWidget {
  const _ConfirmStep({required this.l10n});

  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    final c = Get.find<SetupController>();
    final s = c.setup.value;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(l10n.setupConfirmHint),
        const SizedBox(height: 12),
        Text('${l10n.setupStepBaseFiat}: ${s?.baseFiatCurrency ?? '—'}'),
        Text(
          '${l10n.setupStepPackages}: ${s?.activePackageCount ?? 0} active',
        ),
        const SizedBox(height: 16),
        Obx(
          () => FilledButton(
            onPressed: c.busy.value ? null : c.complete,
            child: c.busy.value
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : Text(l10n.setupFinish),
          ),
        ),
      ],
    );
  }
}
