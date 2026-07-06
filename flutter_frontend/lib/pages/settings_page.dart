import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../widgets/admin_page_scaffold.dart';

class SettingsPage extends StatelessWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return AdminPageScaffold(
      title: 'Settings',
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: const [
          SwitchListTile(
            key: Key('setting-notifications'),
            title: Text('Email notifications'),
            value: true,
            onChanged: null,
          ),
          SwitchListTile(
            key: Key('setting-2fa'),
            title: Text('Two-factor authentication'),
            value: false,
            onChanged: null,
          ),
          ListTile(
            leading: Icon(FluentIcons.color_background_24_regular),
            title: Text('Theme'),
            trailing: Text('System'),
          ),
        ],
      ),
    );
  }
}
