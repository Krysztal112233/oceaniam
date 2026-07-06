import 'package:flutter/material.dart';

import '../../widgets/placeholder_page.dart';

/// 应用级审计日志（按 audit_type 过滤）。
///
/// 后端：GET /tenants/{tid}/applications/{aid}/audits
class ApplicationAuditsPage extends StatelessWidget {
  final String tenantId;
  final String applicationId;

  const ApplicationAuditsPage({
    super.key,
    required this.tenantId,
    required this.applicationId,
  });

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Application audits',
      description: 'App-scoped audit log, filterable by audit_type.',
    );
  }
}
