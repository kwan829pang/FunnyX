/// Runtime config loaded by each app (typically via `envied`).
class AppConfig {
  const AppConfig({
    required this.appName,
    required this.apiBaseUrl,
    this.defaultPartnerId = 'partner_2001',
    this.platformOauthClientId = 'platform_partner_client',
    this.platformOauthPartnerRedirect =
        'http://127.0.0.1:18102/oauth/callback',
  });

  final String appName;
  final String apiBaseUrl;

  /// Partner id for Client Web → Partner OAuth (Path B).
  final String defaultPartnerId;

  /// Platform OAuth client_id Partners use (Path C / platform IdP).
  final String platformOauthClientId;

  /// Partner redirect_uri registered on platform OAuth (demo: company-a).
  final String platformOauthPartnerRedirect;
}
