import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../widgets/expandable_list_card.dart';
import 'application_expanded_panel.dart';

class ApplicationCard extends StatelessWidget {
  final Application application;
  final String tenantId;
  final bool isExpanded;
  final VoidCallback? onExpand;

  const ApplicationCard({
    super.key,
    required this.application,
    required this.tenantId,
    required this.isExpanded,
    this.onExpand,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return ExpandableListCard(
      leading: CircleAvatar(
        backgroundColor: theme.colorScheme.secondaryContainer,
        child: Icon(
          FluentIcons.app_folder_24_regular,
          color: theme.colorScheme.onSecondaryContainer,
        ),
      ),
      title: Text(application.id),
      subtitle: application.comment != null && application.comment!.isNotEmpty
          ? Text(application.comment!)
          : null,
      isExpanded: isExpanded,
      onExpand: onExpand,
      expandedChild: ApplicationExpandedPanel(
        tenantId: tenantId,
        applicationId: application.id,
      ),
    );
  }
}
