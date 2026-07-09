import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import 'application_expanded_panel.dart';

class ApplicationCard extends StatelessWidget {
  final Application application;
  final String tenantId;
  final bool isExpanded;
  final VoidCallback? onExpand;
  final VoidCallback? onUsers;

  const ApplicationCard({
    super.key,
    required this.application,
    required this.tenantId,
    required this.isExpanded,
    this.onExpand,
    this.onUsers,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final expandEnabled = onExpand != null;
    final detailEnabled = onUsers != null;

    return Card(
      margin: const EdgeInsets.only(bottom: 16),
      clipBehavior: Clip.antiAlias,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          ListTile(
            leading: CircleAvatar(
              backgroundColor: theme.colorScheme.secondaryContainer,
              child: Icon(
                FluentIcons.app_folder_24_regular,
                color: theme.colorScheme.onSecondaryContainer,
              ),
            ),
            title: Text(application.id),
            subtitle:
                application.comment != null && application.comment!.isNotEmpty
                ? Text(application.comment!)
                : null,
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (expandEnabled)
                  Padding(
                    padding: const EdgeInsets.only(left: 4),
                    child: Icon(
                      isExpanded
                          ? FluentIcons.chevron_up_24_regular
                          : FluentIcons.chevron_down_24_regular,
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  )
                else if (detailEnabled)
                  Padding(
                    padding: const EdgeInsets.only(left: 4),
                    child: Icon(
                      FluentIcons.more_horizontal_24_regular,
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
              ],
            ),
            onTap: expandEnabled ? onExpand : onUsers,
          ),
          AnimatedCrossFade(
            firstChild: const SizedBox.shrink(),
            secondChild: ApplicationExpandedPanel(
              tenantId: tenantId,
              applicationId: application.id,
            ),
            crossFadeState: isExpanded
                ? CrossFadeState.showSecond
                : CrossFadeState.showFirst,
            duration: const Duration(milliseconds: 220),
            firstCurve: Curves.easeInOut,
            secondCurve: Curves.easeInOut,
            sizeCurve: Curves.easeInOut,
            alignment: Alignment.topCenter,
          ),
        ],
      ),
    );
  }
}
