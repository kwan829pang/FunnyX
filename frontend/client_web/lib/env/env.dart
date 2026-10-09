import 'package:envied/envied.dart';

part 'env.g.dart';

@Envied(path: '.env', useConstantCase: false)
abstract class Env {
  @EnviedField(varName: 'API_BASE_URL', defaultValue: 'http://127.0.0.1:8080')
  static const String apiBaseUrl = _Env.apiBaseUrl;

  @EnviedField(varName: 'APP_NAME', defaultValue: 'FunnyX')
  static const String appName = _Env.appName;

  @EnviedField(varName: 'DEFAULT_PARTNER_ID', defaultValue: 'partner_2001')
  static const String defaultPartnerId = _Env.defaultPartnerId;

  @EnviedField(
    varName: 'PLATFORM_OAUTH_CLIENT_ID',
    defaultValue: 'platform_partner_client',
  )
  static const String platformOauthClientId = _Env.platformOauthClientId;

  @EnviedField(
    varName: 'PLATFORM_OAUTH_PARTNER_REDIRECT',
    defaultValue: 'http://127.0.0.1:18102/oauth/callback',
  )
  static const String platformOauthPartnerRedirect =
      _Env.platformOauthPartnerRedirect;
}
