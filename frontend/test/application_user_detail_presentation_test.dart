import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:frontend/pages/applications/application_user_detail_presentation.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

void main() {
  const user = ApplicationUser(id: 'user-1', nickname: 'Alice');

  Future<void> pumpLauncher(WidgetTester tester, Size viewport) {
    return tester.pumpWidget(
      ProviderScope(
        child: MaterialApp(
          home: MediaQuery(
            data: MediaQueryData(size: viewport),
            child: Builder(
              builder: (context) => Scaffold(
                body: FilledButton(
                  onPressed: () => showApplicationUserDetail(
                    context,
                    tenantId: 'tenant-1',
                    applicationId: 'application-1',
                    user: user,
                  ),
                  child: const Text('Manage user'),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  testWidgets('opens a 520dp side sheet on wide screens', (tester) async {
    await pumpLauncher(tester, const Size(1200, 800));

    await tester.tap(find.text('Manage user'));
    await tester.pumpAndSettle();

    expect(
      find.byKey(const Key('application-user-detail-sheet')),
      findsOneWidget,
    );
    expect(
      tester
          .getSize(find.byKey(const Key('application-user-detail-sheet')))
          .width,
      520,
    );
  });

  testWidgets('opens a side sheet at the 900dp breakpoint', (tester) async {
    await pumpLauncher(tester, const Size(900, 800));

    await tester.tap(find.text('Manage user'));
    await tester.pumpAndSettle();

    expect(
      find.byKey(const Key('application-user-detail-sheet')),
      findsOneWidget,
    );
  });

  testWidgets('opens a full page on narrow screens', (tester) async {
    await pumpLauncher(tester, const Size(600, 800));

    await tester.tap(find.text('Manage user'));
    await tester.pumpAndSettle();

    expect(find.text('User status'), findsOneWidget);
    expect(
      find.byKey(const Key('application-user-detail-sheet')),
      findsNothing,
    );
  });
}
