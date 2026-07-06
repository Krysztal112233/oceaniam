import 'package:flutter/material.dart';

import '../../widgets/placeholder_page.dart';

/// 应用级统计 + 新用户趋势。
///
/// 后端：
/// - GET /tenants/{tid}/applications/{aid}/statistics
/// - GET /tenants/{tid}/applications/{aid}/statistics/trends
class ApplicationStatisticsPage extends StatelessWidget {
  final String tenantId;
  final String applicationId;

  const ApplicationStatisticsPage({
    super.key,
    required this.tenantId,
    required this.applicationId,
  });

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Statistics',
      description: 'App-level totals and new-user trends.',
    );
  }
}
