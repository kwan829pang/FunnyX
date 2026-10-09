import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import '../../l10n/app_localizations.dart';
import '../../services/oauth_flow.dart';

/// Side-by-side summary of Path B vs Path C + Partner OAuth start action.
class OauthComparePanel extends StatelessWidget {
  const OauthComparePanel({super.key, this.showPartnerButton = true});

  final bool showPartnerButton;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final oauth = Get.find<OauthFlowService>();
    final platformUrl = oauth.platformOauthAuthorizeUri().toString();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(l10n.oauthCompareTitle, style: theme.textTheme.titleMedium),
        const SizedBox(height: 8),
        _pathCard(
          context,
          title: l10n.oauthPathBTitle,
          body: l10n.oauthPathBBody,
        ),
        const SizedBox(height: 8),
        _pathCard(
          context,
          title: l10n.oauthPathCTitle,
          body: l10n.oauthPathCBody,
        ),
        if (showPartnerButton) ...[
          const SizedBox(height: 16),
          OutlinedButton.icon(
            onPressed: () async {
              try {
                await oauth.startPartnerOauth();
              } catch (e) {
                toastification.show(
                  title: Text('$e'),
                  type: ToastificationType.error,
                  autoCloseDuration: const Duration(seconds: 4),
                );
              }
            },
            icon: const Icon(Icons.sports_esports_outlined),
            label: Text(l10n.oauthContinuePartner),
          ),
        ],
        const SizedBox(height: 12),
        Text(l10n.oauthPlatformAuthorizeHint, style: theme.textTheme.bodySmall),
        const SizedBox(height: 4),
        SelectableText(
          platformUrl,
          style: theme.textTheme.bodySmall?.copyWith(
            fontFamily: 'monospace',
            fontSize: 11,
          ),
        ),
        Align(
          alignment: Alignment.centerLeft,
          child: TextButton.icon(
            onPressed: () async {
              await Clipboard.setData(ClipboardData(text: platformUrl));
              toastification.show(
                title: Text(l10n.oauthCopied),
                type: ToastificationType.success,
                autoCloseDuration: const Duration(seconds: 2),
              );
            },
            icon: const Icon(Icons.copy, size: 16),
            label: Text(l10n.oauthCopyAuthorize),
          ),
        ),
      ],
    );
  }

  Widget _pathCard(
    BuildContext context, {
    required String title,
    required String body,
  }) {
    final theme = Theme.of(context);
    return DecoratedBox(
      decoration: BoxDecoration(
        border: Border.all(color: theme.dividerColor),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: theme.textTheme.titleSmall),
            const SizedBox(height: 4),
            Text(body, style: theme.textTheme.bodySmall),
          ],
        ),
      ),
    );
  }
}
