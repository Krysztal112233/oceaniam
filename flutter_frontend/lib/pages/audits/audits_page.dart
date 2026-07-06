import 'package:flutter/material.dart';

import '../../widgets/placeholder_page.dart';

/// 平台级审计日志（全量，audit_type 过滤）。
///
/// 后端：GET /audits（TenantRead）
class AuditsPage extends StatelessWidget {
  const AuditsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Audits',
      description: 'Platform-wide audit log, filterable by audit_type.',
    );
  }
}
