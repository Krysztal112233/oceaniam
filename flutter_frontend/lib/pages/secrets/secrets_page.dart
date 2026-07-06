import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../widgets/placeholder_page.dart';

/// 平台级 API Secrets 列表 + 创建（一次性明文）。
///
/// 后端：
/// - GET /secrets（SecretRead）
/// - POST /secrets（SecretCreate，返回一次性明文）
class SecretsPage extends StatelessWidget {
  const SecretsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'API secrets',
      description:
          'Platform-level secrets. Create returns the unmasked value once.',
      actions: [
        FilledButton.icon(
          key: const Key('create-secret'),
          onPressed: () {
            // TODO: POST /secrets
          },
          icon: const Icon(FluentIcons.add_24_regular),
          label: const Text('New secret'),
        ),
      ],
    );
  }
}
