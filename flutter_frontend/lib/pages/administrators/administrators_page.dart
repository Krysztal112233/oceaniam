import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../widgets/placeholder_page.dart';

/// 平台管理员列表（创建返回初始密码）。
///
/// 后端：
/// - GET /administrators（AdministratorRead）
/// - POST /administrators（AdministratorCreate）
class AdministratorsPage extends StatelessWidget {
  const AdministratorsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Administrators',
      description: 'Platform admins. Create returns the initial password once.',
      actions: [
        FilledButton.icon(
          key: const Key('create-admin'),
          onPressed: () {
            // TODO: POST /administrators
          },
          icon: const Icon(FluentIcons.person_add_24_regular),
          label: const Text('New admin'),
        ),
      ],
    );
  }
}
