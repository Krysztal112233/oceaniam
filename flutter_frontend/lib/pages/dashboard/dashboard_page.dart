import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../widgets/admin_page_scaffold.dart';
import '../../widgets/stat_grid.dart';

/// 平台仪表盘。
///
/// 后端：GET /statistics（5 个总数）+ GET /statistics/trends（4 条趋势）。
/// 当前 [StatGrid] 使用占位数据，后续替换为真实统计。
class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const AdminPageScaffold(title: 'Dashboard', child: StatGrid());
  }
}
