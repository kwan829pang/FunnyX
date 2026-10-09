import 'package:flutter_test/flutter_test.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

void main() {
  setUp(() {
    Get.reset();
  });

  test('SessionController tracks auth token', () {
    final session = SessionController();
    expect(session.isAuthenticated, isFalse);
    session.setToken('jwt-demo');
    expect(session.isAuthenticated, isTrue);
    expect(session.token, 'jwt-demo');
    session.clear();
    expect(session.isAuthenticated, isFalse);
  });

  test('MockAuthDataProvider issues a session token', () async {
    final auth = MockAuthDataProvider(role: 'admin');
    final session = await auth.login(username: 'admin', password: 'demo');
    expect(session.token, isNotEmpty);
    expect(session.accountId, 'admin');
  });

  test('SessionToken.fromAuthJson parses Client Center login payload', () {
    final session = SessionToken.fromAuthJson({
      'access_token': 'sts_abc',
      'refresh_token': 'str_xyz',
      'end_user_id': 1,
      'username': 'demo_user',
      'account_id': 'demo_user',
      'expires_at_ms': 123,
    });
    expect(session.token, 'sts_abc');
    expect(session.refreshToken, 'str_xyz');
    expect(session.endUserId, 1);
    expect(session.accountId, 'demo_user');
  });
}
