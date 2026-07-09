import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import 'application_tab_placeholder.dart';

class ApplicationSecretsTab extends StatelessWidget {
  final String applicationId;

  const ApplicationSecretsTab({super.key, required this.applicationId});

  @override
  Widget build(BuildContext context) {
    return ApplicationTabPlaceholder(
      icon: FluentIcons.key_24_regular,
      title: 'Secrets',
      description: 'Manage secret bindings for $applicationId.',
    );
  }
}
