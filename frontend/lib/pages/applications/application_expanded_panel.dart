import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../widgets/segmented_expand_panel.dart';
import 'application_tab_contents.dart';
import 'create_user_dialog.dart';

class ApplicationExpandedPanel extends StatelessWidget {
  final String tenantId;
  final String applicationId;

  const ApplicationExpandedPanel({
    super.key,
    required this.tenantId,
    required this.applicationId,
  });

  @override
  Widget build(BuildContext context) {
    return SegmentedExpandPanel(
      initialIndex: 1,
      tabs: [
        ExpandPanelTab(
          icon: FluentIcons.info_24_regular,
          label: 'Overview',
          builder: (_) => ApplicationOverviewTab(
            tenantId: tenantId,
            applicationId: applicationId,
          ),
        ),
        ExpandPanelTab(
          icon: FluentIcons.people_24_regular,
          label: 'Users',
          builder: (ctx) => ApplicationUsersTab(
            tenantId: tenantId,
            applicationId: applicationId,
            fillAvailable: false,
            action: FilledButton.tonalIcon(
              onPressed: () {
                showDialog<void>(
                  context: ctx,
                  builder: (_) => CreateUserDialog(
                    tenantId: tenantId,
                    applicationId: applicationId,
                  ),
                );
              },
              icon: const Icon(FluentIcons.add_24_regular),
              label: const Text('New user'),
            ),
          ),
        ),
        ExpandPanelTab(
          icon: FluentIcons.settings_24_regular,
          label: 'Settings',
          builder: (_) => ApplicationSettingsTab(
            tenantId: tenantId,
            applicationId: applicationId,
          ),
        ),
      ],
    );
  }
}
