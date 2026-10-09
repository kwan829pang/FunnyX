import 'package:funnyx_common/funnyx_common.dart';
import 'package:url_launcher/url_launcher.dart';

/// Client Web OAuth helpers for both identity directions.
///
/// **Path B — Partner → Platform:** user already at Partner; Client Web starts
/// Partner OAuth via Client Center → STS broker → callback with `sts_…`.
///
/// **Path C — Platform → Partner:** user registered on Client Web; Partner
/// redirects to platform `/v1/oauth/authorize` (STS IdP). Client Web does not
/// start Path C; it exposes the authorize URL Partners embed.
class OauthFlowService {
  OauthFlowService(this.config);

  final AppConfig config;

  /// Browser return URL after Partner OAuth completes (Client Center → here).
  String clientWebCallbackUri() {
    final base = Uri.base;
    return Uri(
      scheme: base.scheme,
      host: base.host,
      port: base.hasPort ? base.port : null,
      path: '/oauth/callback',
    ).toString();
  }

  /// Client Center Partner OAuth start (Path B).
  Uri partnerOauthStartUri({String? partnerId, String? redirectUri}) {
    final id = partnerId ?? config.defaultPartnerId;
    final redirect = redirectUri ?? clientWebCallbackUri();
    final base = config.apiBaseUrl.replaceAll(RegExp(r'/$'), '');
    return Uri.parse('$base/v1/client/oauth/partner/$id/start').replace(
      queryParameters: {'redirect_uri': redirect},
    );
  }

  /// Platform OAuth authorize URL for Partners (Path C / platform IdP).
  Uri platformOauthAuthorizeUri({String? state}) {
    final base = config.apiBaseUrl.replaceAll(RegExp(r'/$'), '');
    return Uri.parse('$base/v1/oauth/authorize').replace(
      queryParameters: {
        'response_type': 'code',
        'client_id': config.platformOauthClientId,
        'redirect_uri': config.platformOauthPartnerRedirect,
        'state': state ?? 'partner_demo',
        'scope': 'openid profile',
      },
    );
  }

  Future<void> startPartnerOauth({String? partnerId}) async {
    final uri = partnerOauthStartUri(partnerId: partnerId);
    final ok = await launchUrl(uri, webOnlyWindowName: '_self');
    if (!ok) {
      throw StateError('Could not open Partner OAuth: $uri');
    }
  }

  /// Parse tokens from OAuth callback query (CC/STS redirect).
  static SessionToken? sessionFromCallbackQuery(Map<String, String> query) {
    final access = query['access_token'];
    if (access == null || access.isEmpty) {
      return null;
    }
    return SessionToken(
      token: access,
      refreshToken: query['refresh_token'],
      accountId: query['username'] ?? query['end_user_id'] ?? '',
      endUserId: int.tryParse(query['end_user_id'] ?? ''),
      username: query['username'],
    );
  }
}
