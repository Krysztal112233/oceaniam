import 'package:flutter/material.dart';

import '../../widgets/placeholder_page.dart';

/// 应用详情（tabs：概览 / 用户 / 角色 / Secrets / 统计 / 审计 / 配置）。
///
/// 后端：
/// - GET /tenants/{tid}/applications/{aid}（AdminJwtOrAppSecret）
/// - PATCH /tenants/{tid}/applications/{aid}（ApplicationPatch）
/// - DELETE /tenants/{tid}/applications/{aid}（ApplicationDelete）
/// - GET /tenants/{tid}/applications/{aid}/configuration
/// - PATCH /tenants/{tid}/applications/{aid}/configuration（ApplicationConfigurationPatch）
class ApplicationDetailPage extends StatelessWidget {
  final String tenantId;
  final String applicationId;

  const ApplicationDetailPage({
    super.key,
    required this.tenantId,
    required this.applicationId,
  });

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Application $applicationId',
      description:
          'Overview, users, roles, secrets, statistics, audits, configuration.',
    );
  }
}
