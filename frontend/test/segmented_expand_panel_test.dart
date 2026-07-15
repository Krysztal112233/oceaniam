import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:frontend/widgets/segmented_expand_panel.dart';

void main() {
  const tabs = [
    ExpandPanelTab(
      icon: Icons.info_outline,
      label: 'Overview',
      builder: _overview,
    ),
    ExpandPanelTab(icon: Icons.people_outline, label: 'Users', builder: _users),
    ExpandPanelTab(
      icon: Icons.tune,
      label: 'Configuration',
      builder: _configuration,
    ),
    ExpandPanelTab(
      icon: Icons.settings_outlined,
      label: 'Settings',
      builder: _settings,
    ),
  ];

  testWidgets('uses a section picker when tab labels would be clipped', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: SizedBox(
              width: 360,
              child: SegmentedExpandPanel(tabs: tabs),
            ),
          ),
        ),
      ),
    );

    expect(
      find.byKey(const Key('expand-panel-section-picker')),
      findsOneWidget,
    );
    expect(find.text('Overview content'), findsOneWidget);

    await tester.tap(find.byKey(const Key('expand-panel-section-picker')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Configuration').last);
    await tester.pumpAndSettle();

    expect(find.text('Configuration content'), findsOneWidget);
  });

  testWidgets('keeps segmented navigation when all tabs fit', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: SizedBox(
              width: 700,
              child: SegmentedExpandPanel(tabs: tabs),
            ),
          ),
        ),
      ),
    );

    expect(find.byType(SegmentedButton<int>), findsOneWidget);
    expect(find.byKey(const Key('expand-panel-section-picker')), findsNothing);
  });
}

Widget _overview(BuildContext context) => const Text('Overview content');
Widget _users(BuildContext context) => const Text('Users content');
Widget _configuration(BuildContext context) =>
    const Text('Configuration content');
Widget _settings(BuildContext context) => const Text('Settings content');
