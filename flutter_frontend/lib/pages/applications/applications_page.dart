import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../widgets/placeholder_page.dart';

/// 应用分页列表（租户内）。
///
/// 后端：
/// - GET /tenants/{tid}/applications（ApplicationRead）
/// - POST /tenants/{tid}/applications（ApplicationCreate）
class ApplicationsPage extends StatelessWidget {
  final String tenantId;

  const ApplicationsPage({super.key, required this.tenantId});

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Applications',
      description: 'Paginated application list under tenant $tenantId.',
      actions: [
        FilledButton.icon(
          key: const Key('create-application'),
          onPressed: () {
            // TODO: POST /tenants/{tid}/applications
          },
          icon: const Icon(FluentIcons.add_24_regular),
          label: const Text('New application'),
        ),
      ],
    );
  }
}
