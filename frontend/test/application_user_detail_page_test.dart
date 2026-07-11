import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:frontend/pages/applications/application_user_detail_page.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

void main() {
  Widget buildSubject() => ProviderScope(
    child: MaterialApp(
      home: SizedBox(
        width: 1200,
        height: 800,
        child: ApplicationUserDetailPage(
          tenantId: 'tenant-1',
          applicationId: 'application-1',
          user: const ApplicationUser(id: 'user-1', nickname: 'Alice'),
        ),
      ),
    ),
  );

  testWidgets('shows user management actions', (tester) async {
    await tester.pumpWidget(buildSubject());

    expect(find.text('User status'), findsOneWidget);
    expect(find.text('Change nickname'), findsOneWidget);
    expect(find.text('Change password'), findsOneWidget);
    expect(find.text('More'), findsOneWidget);
    final moreButton = tester.widget<FilledButton>(
      find.ancestor(of: find.text('More'), matching: find.byType(FilledButton)),
    );
    expect(moreButton.onPressed, isNotNull);
    expect(find.text('Delete user'), findsNothing);
  });

  testWidgets('opens nickname and password dialogs', (tester) async {
    await tester.pumpWidget(buildSubject());

    await tester.tap(find.text('Change nickname'));
    await tester.pumpAndSettle();
    expect(find.text('Change Nickname'), findsOneWidget);

    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Change password'));
    await tester.pumpAndSettle();
    expect(find.text('Change Password'), findsOneWidget);
  });

  testWidgets('opens the delete confirmation from More', (tester) async {
    await tester.pumpWidget(buildSubject());

    await tester.tap(find.text('More'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Delete user'));
    await tester.pumpAndSettle();

    expect(find.text('Delete User'), findsOneWidget);
    expect(find.text('To confirm, type the following words:'), findsOneWidget);
    expect(find.text('Delete user'), findsOneWidget);
  });
}
