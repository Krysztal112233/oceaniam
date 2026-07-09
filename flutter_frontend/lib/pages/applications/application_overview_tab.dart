import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import 'application_tab_placeholder.dart';

class ApplicationOverviewTab extends StatelessWidget {
  final String applicationId;

  const ApplicationOverviewTab({super.key, required this.applicationId});

  @override
  Widget build(BuildContext context) {
    return ApplicationTabPlaceholder(
      icon: FluentIcons.info_24_regular,
      title: 'Overview',
      description: 'Application metadata and settings for $applicationId.',
    );
  }
}
