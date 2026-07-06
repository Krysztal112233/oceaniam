import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/dashboard_providers.dart';
import '../../widgets/admin_page_scaffold.dart';
import '../../widgets/stat_grid.dart';
import '../../widgets/trend_chart_card.dart';
import '../../widgets/trend_filter_bar.dart';

/// 平台仪表盘。
///
/// 顶部：概览统计卡片（5 张，不受筛选影响）。
/// 中部：趋势筛选条（聚合粒度 + 时间窗口）。
/// 底部：4 张独立趋势折线图（Tenants / Applications / Users / Administrators），
/// 2 列网格（窄屏单列）。
class DashboardPage extends ConsumerWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final trends = ref.watch(dashboardTrendsProvider);
    final isWide = MediaQuery.of(context).size.width >= 900;
    final crossAxis = isWide ? 2 : 1;

    return AdminPageScaffold(
      title: 'Dashboard',
      child: ListView(
        padding: const EdgeInsets.symmetric(vertical: 8),
        children: [
          const StatGrid(),
          const TrendFilterBar(),
          const SizedBox(height: 8),
          trends.when(
            loading: () => const _TrendSkeleton(),
            error: (err, stack) => _TrendError(
              message: 'Failed to load trends: $err',
              onRetry: () => ref.invalidate(dashboardTrendsProvider),
            ),
            data: (t) => _TrendGrid(trends: t, crossAxis: crossAxis),
          ),
        ],
      ),
    );
  }
}

class _TrendGrid extends StatelessWidget {
  final PlatformTrends trends;
  final int crossAxis;

  const _TrendGrid({required this.trends, required this.crossAxis});

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final cards = <TrendChartCard>[
      TrendChartCard(
        title: 'Tenants',
        points: trends.tenants,
        color: scheme.primary,
      ),
      TrendChartCard(
        title: 'Applications',
        points: trends.applications,
        color: scheme.secondary,
      ),
      TrendChartCard(
        title: 'Users',
        points: trends.users,
        color: scheme.tertiary,
      ),
      TrendChartCard(
        title: 'Administrators',
        points: trends.administrators,
        color: scheme.error,
      ),
    ];

    return GridView.count(
      crossAxisCount: crossAxis,
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
      crossAxisSpacing: 16,
      mainAxisSpacing: 16,
      childAspectRatio: crossAxis == 2 ? 1.6 : 2.2,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      children: cards,
    );
  }
}

class _TrendSkeleton extends StatelessWidget {
  const _TrendSkeleton();

  @override
  Widget build(BuildContext context) {
    final isWide = MediaQuery.of(context).size.width >= 900;
    return GridView.count(
      crossAxisCount: isWide ? 2 : 1,
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
      crossAxisSpacing: 16,
      mainAxisSpacing: 16,
      childAspectRatio: isWide ? 1.6 : 2.2,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      children: List.generate(4, (_) {
        return Card(
          elevation: 0,
          shape: RoundedRectangleBorder(
            side: BorderSide(color: Theme.of(context).dividerColor),
            borderRadius: BorderRadius.circular(12),
          ),
          child: const Center(
            child: SizedBox(
              width: 28,
              height: 28,
              child: CircularProgressIndicator(strokeWidth: 2.5),
            ),
          ),
        );
      }),
    );
  }
}

class _TrendError extends StatelessWidget {
  final String message;
  final VoidCallback onRetry;
  const _TrendError({required this.message, required this.onRetry});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
      child: Card(
        elevation: 0,
        shape: RoundedRectangleBorder(
          side: BorderSide(color: Theme.of(context).dividerColor),
          borderRadius: BorderRadius.circular(12),
        ),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Icon(
                Icons.error_outline,
                color: Theme.of(context).colorScheme.error,
              ),
              const SizedBox(width: 12),
              Expanded(child: Text(message)),
              TextButton(onPressed: onRetry, child: const Text('Retry')),
            ],
          ),
        ),
      ),
    );
  }
}
