import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:frontend/pages/applications/configuration/application_configuration_editor.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

void main() {
  const configuration = ApplicationConfiguration(
    auth: AuthConfiguration(
      token: TokenConfiguration(issuer: 'OceanIAM', audience: ['OceanIAM']),
      password: PasswordConfiguration(
        argon2: Argon2Configuration(mCost: 12288, tCost: 3, pCost: 1),
      ),
    ),
    registration: RegistrationConfiguration(enabled: false),
  );

  Widget buildSubject(SaveApplicationConfiguration onSave) {
    return MaterialApp(
      home: Scaffold(
        body: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: ApplicationConfigurationEditor(
            configuration: configuration,
            onSave: onSave,
          ),
        ),
      ),
    );
  }

  testWidgets('shows every configuration field and saves a minimal patch', (
    tester,
  ) async {
    PatchApplicationConfiguration? savedPatch;
    await tester.pumpWidget(
      buildSubject((patch) async {
        savedPatch = patch;
        return true;
      }),
    );

    expect(find.text('Token'), findsOneWidget);
    expect(find.text('Password hashing'), findsOneWidget);
    expect(find.text('Registration'), findsOneWidget);
    expect(find.text('Read only'), findsOneWidget);
    expect(
      find.byKey(const Key('configuration-argon2-memory')),
      findsOneWidget,
    );
    expect(find.byKey(const Key('configuration-argon2-time')), findsOneWidget);
    expect(
      find.byKey(const Key('configuration-argon2-parallelism')),
      findsOneWidget,
    );
    TextFormField argonField(String key) => tester.widget<TextFormField>(
      find.descendant(
        of: find.byKey(Key(key)),
        matching: find.byType(TextFormField),
      ),
    );

    expect(argonField('configuration-argon2-memory').initialValue, '12288');
    expect(argonField('configuration-argon2-time').initialValue, '3');
    expect(argonField('configuration-argon2-parallelism').initialValue, '1');

    final saveButton = tester.widget<FilledButton>(
      find.byKey(const Key('configuration-save')),
    );
    expect(saveButton.onPressed, isNull);

    await tester.enterText(
      find.byKey(const Key('configuration-token-issuer')),
      'Example',
    );
    await tester.enterText(
      find.byKey(const Key('configuration-audience-input')),
      'example-api',
    );
    await tester.tap(find.byKey(const Key('configuration-audience-add')));
    await tester.ensureVisible(
      find.byKey(const Key('configuration-registration-enabled')),
    );
    await tester.tap(
      find.byKey(const Key('configuration-registration-enabled')),
    );
    await tester.pump();
    await tester.ensureVisible(find.byKey(const Key('configuration-save')));
    await tester.tap(find.byKey(const Key('configuration-save')));
    await tester.pump();

    expect(savedPatch?.toJson(), {
      'auth': {
        'token': {
          'issuer': 'Example',
          'audience': ['OceanIAM', 'example-api'],
        },
      },
      'registration': {'enabled': true},
    });
  });

  testWidgets('requires at least one token audience', (tester) async {
    var saveCount = 0;
    await tester.pumpWidget(
      buildSubject((_) async {
        saveCount += 1;
        return true;
      }),
    );

    final chip = tester.widget<InputChip>(
      find.byKey(const ValueKey('audience-OceanIAM')),
    );
    chip.onDeleted!();
    await tester.pump();
    await tester.tap(find.byKey(const Key('configuration-save')));
    await tester.pump();

    expect(find.text('At least one audience is required.'), findsOneWidget);
    expect(saveCount, 0);
  });
}
