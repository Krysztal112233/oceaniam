import 'package:flutter/material.dart';

import '../../widgets/placeholder_page.dart';

/// Secret 详情 + 绑定关系管理。
///
/// 后端：
/// - GET /secrets/{sid}（SecretRead）
/// - DELETE /secrets/{sid}（SecretDelete）
/// - POST /secrets/{sid}/bindings（SecretCreate）
/// - DELETE /secrets/{sid}/bindings/{application_id}（SecretDelete）
class SecretDetailPage extends StatelessWidget {
  final String secretId;

  const SecretDetailPage({super.key, required this.secretId});

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Secret $secretId',
      description: 'Masked secret, bind/unbind to applications.',
    );
  }
}
