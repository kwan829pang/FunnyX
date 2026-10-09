import 'package:client_web/app.dart';
import 'package:client_web/bindings/app_binding.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

void main() {
  setUp(() {
    Get.reset();
    AppBinding(
      config: const AppConfig(
        appName: 'FunnyX',
        apiBaseUrl: 'http://127.0.0.1:8080',
      ),
    ).dependencies();
  });

  testWidgets('login page is shown without a token', (tester) async {
    await tester.pumpWidget(const ClientApp());
    await tester.pumpAndSettle();
    expect(find.text('Login'), findsWidgets);
  });
}
