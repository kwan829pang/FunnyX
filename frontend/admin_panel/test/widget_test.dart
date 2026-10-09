import 'package:admin_panel/app.dart';
import 'package:admin_panel/bindings/app_binding.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:funnyx_common/funnyx_common.dart';
import 'package:get/get.dart';

void main() {
  setUp(() {
    Get.reset();
    AppBinding(
      config: const AppConfig(
        appName: 'FunnyX Admin',
        apiBaseUrl: 'http://127.0.0.1:18300',
      ),
    ).dependencies();
  });

  testWidgets('login page is shown without a token', (tester) async {
    await tester.pumpWidget(const AdminApp());
    await tester.pumpAndSettle();
    expect(find.text('Login'), findsWidgets);
  });
}
