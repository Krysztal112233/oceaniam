import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../widgets/placeholder_page.dart';

/// ORBAC 应用角色列表（系统 + 自定义）。
///
/// 后端：
/// - GET /tenants/{tid}/applications/{aid}/roles
/// - POST /tenants/{tid}/applications/{aid}/roles
class ApplicationRolesPage extends StatelessWidget {
  final String tenantId;
  final String applicationId;

  const ApplicationRolesPage({
    super.key,
    required this.tenantId,
    required this.applicationId,
  });

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Roles',
      description:
          'ORBAC roles: system + custom. Manage permissions and subject assignments.',
      actions: [
        FilledButton.icon(
          key: const Key('create-role'),
          onPressed: () {
            // TODO: POST .../roles
          },
          icon: const Icon(FluentIcons.add_24_regular),
          label: const Text('New role'),
        ),
      ],
    );
  }
}
