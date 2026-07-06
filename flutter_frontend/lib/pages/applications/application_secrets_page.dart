import 'package:flutter/material.dart';

import '../../widgets/placeholder_page.dart';

/// 应用绑定的 Secrets（masked）。
///
/// 后端：
/// - GET /tenants/{tid}/applications/{aid}/secrets（SecretRead）
class ApplicationSecretsPage extends StatelessWidget {
  final String tenantId;
  final String applicationId;

  const ApplicationSecretsPage({
    super.key,
    required this.tenantId,
    required this.applicationId,
  });

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Application secrets',
      description: 'Bound secrets (masked) for this application.',
    );
  }
}
