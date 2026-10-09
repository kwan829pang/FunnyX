import 'package:envied/envied.dart';

part 'env.g.dart';

@Envied(path: '.env', useConstantCase: false)
abstract class Env {
  @EnviedField(varName: 'API_BASE_URL', defaultValue: 'http://127.0.0.1:8080')
  static const String apiBaseUrl = _Env.apiBaseUrl;

  @EnviedField(varName: 'APP_NAME', defaultValue: 'FunnyX Admin')
  static const String appName = _Env.appName;
}
