import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../widgets/placeholder_page.dart';

/// 应用用户列表 + 搜索 + 创建。
///
/// 后端：
/// - GET /tenants/{tid}/applications/{aid}/users
/// - GET /tenants/{tid}/applications/{aid}/users/search
/// - POST /tenants/{tid}/applications/{aid}/users
class ApplicationUsersPage extends StatelessWidget {
  final String tenantId;
  final String applicationId;

  const ApplicationUsersPage({
    super.key,
    required this.tenantId,
    required this.applicationId,
  });

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Application users',
      description: 'List, search, create users under this application.',
      actions: [
        FilledButton.icon(
          key: const Key('create-app-user'),
          onPressed: () {
            // TODO: POST .../users
          },
          icon: const Icon(FluentIcons.person_add_24_regular),
          label: const Text('New user'),
        ),
      ],
    );
  }
}
