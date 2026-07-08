import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../theme/theme_controller.dart';
import '../widgets/admin_page_scaffold.dart';

class SettingsPage extends ConsumerWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeMode = ref.watch(themeControllerProvider);

    return AdminPageScaffold(
      title: 'Settings',
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          const SwitchListTile(
            key: Key('setting-notifications'),
            title: Text('Email notifications'),
            value: true,
            onChanged: null,
          ),
          const SwitchListTile(
            key: Key('setting-2fa'),
            title: Text('Two-factor authentication'),
            value: false,
            onChanged: null,
          ),
          ListTile(
            leading: const Icon(Icons.dark_mode),
            title: const Text('Theme'),
            trailing: SegmentedButton<ThemeMode>(
              segments: const [
                ButtonSegment(
                  value: ThemeMode.system,
                  label: Text('System'),
                  icon: Icon(Icons.brightness_auto),
                ),
                ButtonSegment(
                  value: ThemeMode.light,
                  label: Text('Light'),
                  icon: Icon(Icons.brightness_5),
                ),
                ButtonSegment(
                  value: ThemeMode.dark,
                  label: Text('Dark'),
                  icon: Icon(Icons.brightness_2),
                ),
              ],
              selected: {themeMode},
              onSelectionChanged: (s) {
                ref.read(themeControllerProvider.notifier).setMode(s.first);
              },
            ),
          ),
        ],
      ),
    );
  }
}
