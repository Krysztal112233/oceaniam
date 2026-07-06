import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../widgets/placeholder_page.dart';

/// 租户密钥列表，支持轮换/吊销。
///
/// 后端：
/// - GET /tenants/{tid}/keys（KeyRead）
/// - POST /tenants/{tid}/keys（KeyRotate）
/// - DELETE /tenants/{tid}/keys/{key_id}（KeyRevoke）
class TenantKeysPage extends StatelessWidget {
  final String tenantId;

  const TenantKeysPage({super.key, required this.tenantId});

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Tenant keys',
      description: 'List, rotate, revoke signing keys for this tenant.',
      actions: [
        FilledButton.icon(
          key: const Key('rotate-key'),
          onPressed: () {
            // TODO: POST /tenants/{tid}/keys
          },
          icon: const Icon(FluentIcons.arrow_sync_24_regular),
          label: const Text('Rotate'),
        ),
      ],
    );
  }
}
