import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import 'application_tab_contents.dart';

class ApplicationExpandedPanel extends StatefulWidget {
  final String applicationId;

  const ApplicationExpandedPanel({super.key, required this.applicationId});

  @override
  State<ApplicationExpandedPanel> createState() =>
      _ApplicationExpandedPanelState();
}

class _ApplicationExpandedPanelState extends State<ApplicationExpandedPanel> {
  int _selectedTab = 1;

  static const _tabIcons = [
    FluentIcons.info_24_regular,
    FluentIcons.people_24_regular,
    FluentIcons.key_24_regular,
    FluentIcons.settings_24_regular,
  ];

  static const _tabLabels = ['Overview', 'Users', 'Secrets', 'Settings'];

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    Widget content;
    switch (_selectedTab) {
      case 0:
        content = ApplicationOverviewTab(applicationId: widget.applicationId);
      case 1:
        content = ApplicationUsersTab(
          applicationId: widget.applicationId,
          action: FilledButton.tonalIcon(
            onPressed: () {
              // TODO: implement create user
            },
            icon: const Icon(FluentIcons.add_24_regular),
            label: const Text('New user'),
          ),
        );
      case 2:
        content = ApplicationSecretsTab(applicationId: widget.applicationId);
      default:
        content = ApplicationSettingsTab(
          tenantId: '',
          applicationId: widget.applicationId,
        );
    }

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Divider(height: 1, color: theme.colorScheme.outlineVariant),
        Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                children: [
                  Expanded(
                    child: SegmentedButton<int>(
                      showSelectedIcon: false,
                      segments: _tabLabels
                          .asMap()
                          .entries
                          .map(
                            (e) => ButtonSegment<int>(
                              value: e.key,
                              icon: Icon(_tabIcons[e.key], size: 18),
                              label: Text(e.value),
                            ),
                          )
                          .toList(),
                      selected: {_selectedTab},
                      onSelectionChanged: (v) =>
                          setState(() => _selectedTab = v.first),
                      emptySelectionAllowed: false,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              content,
            ],
          ),
        ),
      ],
    );
  }
}
