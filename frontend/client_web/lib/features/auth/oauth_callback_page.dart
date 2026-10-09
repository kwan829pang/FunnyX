import 'package:flutter/material.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';
import 'package:toastification/toastification.dart';

import '../../controllers/auth_controller.dart';
import '../../l10n/app_localizations.dart';
import '../../services/oauth_flow.dart';

/// Receives Partner OAuth handoff from Client Center (`access_token` query).
class OauthCallbackPage extends StatefulWidget {
  const OauthCallbackPage({super.key});

  @override
  State<OauthCallbackPage> createState() => _OauthCallbackPageState();
}

class _OauthCallbackPageState extends State<OauthCallbackPage> {
  String? _error;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _consume());
  }

  void _consume() {
    final query = <String, String>{...Uri.base.queryParameters};
    for (final e in Get.parameters.entries) {
      final v = e.value;
      if (v != null && v.isNotEmpty) {
        query[e.key] = v;
      }
    }
    // GetX may leave the query on currentRoute.
    final route = Get.currentRoute;
    final qIndex = route.indexOf('?');
    if (qIndex >= 0) {
      query.addAll(Uri.splitQueryString(route.substring(qIndex + 1)));
    }

    if (query['error'] != null && query['error']!.isNotEmpty) {
      setState(() => _error = query['error']);
      return;
    }

    final session = OauthFlowService.sessionFromCallbackQuery(query);
    if (session == null) {
      setState(() => _error = 'missing access_token');
      return;
    }

    final auth = Get.find<AuthController>();
    auth.applyOauthSession(session);
    toastification.show(
      title: Text(AppLocalizations.of(context).oauthSuccess),
      type: ToastificationType.success,
      autoCloseDuration: const Duration(seconds: 2),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return UnauthGate(
      child: Scaffold(
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: _error == null
                ? Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const CircularProgressIndicator(),
                      const SizedBox(height: 16),
                      Text(l10n.oauthCompleting),
                    ],
                  )
                : Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(l10n.oauthFailed),
                      const SizedBox(height: 8),
                      Text(_error!, textAlign: TextAlign.center),
                      const SizedBox(height: 16),
                      FilledButton(
                        onPressed: () => Get.offAllNamed('/login'),
                        child: Text(l10n.login),
                      ),
                    ],
                  ),
          ),
        ),
      ),
    );
  }
}
