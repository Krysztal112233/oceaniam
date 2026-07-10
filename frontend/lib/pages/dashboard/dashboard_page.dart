import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';
import 'package:fl_chart/fl_chart.dart';

import '../../providers/dashboard_providers.dart';
import '../../widgets/admin_page_scaffold.dart';

class DashboardPage extends ConsumerWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final overviewAsync = ref.watch(fetchOverviewProvider);
    final trendsAsync = ref.watch(fetchPlatformTrendsProvider);

    return AdminPageScaffold(
      title: 'Dashboard',
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _SectionHeader(icon: FluentIcons.board_24_regular, title: '系统总览'),
          const SizedBox(height: 12),
          AnimatedSwitcher(
            duration: const Duration(milliseconds: 300),
            child: KeyedSubtree(
              key: ValueKey(overviewAsync),
              child: overviewAsync.when(
                loading: () => const _SkeletonGrid(count: 5),
                error: (e, _) =>
                    _ErrorBox(message: 'Failed to load overview: $e'),
                data: (o) => _OverviewGrid(overview: o),
              ),
            ),
          ),
          const SizedBox(height: 32),
          _SectionHeader(
            icon: FluentIcons.chart_multiple_24_regular,
            title: '统计',
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                _GranularityToggle(),
                const SizedBox(width: 8),
                IconButton(
                  icon: const Icon(FluentIcons.arrow_sync_24_regular),
                  onPressed: () => ref.invalidate(fetchPlatformTrendsProvider),
                  tooltip: '刷新',
                ),
              ],
            ),
          ),
          const SizedBox(height: 12),
          AnimatedSwitcher(
            duration: const Duration(milliseconds: 300),
            child: KeyedSubtree(
              key: ValueKey(trendsAsync),
              child: trendsAsync.when(
                loading: () => const _SkeletonGrid(count: 4),
                error: (e, _) =>
                    _ErrorBox(message: 'Failed to load trends: $e'),
                data: (t) => _TrendsGrid(trends: t),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _SectionHeader extends StatelessWidget {
  final IconData icon;
  final String title;
  final Widget? trailing;

  const _SectionHeader({
    required this.icon,
    required this.title,
    this.trailing,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Row(
      children: [
        Icon(icon, size: 20, color: theme.colorScheme.primary),
        const SizedBox(width: 8),
        Text(title, style: theme.textTheme.titleMedium),
        if (trailing != null) ...[const Spacer(), trailing!],
      ],
    );
  }
}

class _GranularityToggle extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final current = ref.watch(trendsGranularityProvider);
    return SegmentedButton<Granularity>(
      showSelectedIcon: false,
      segments: const [
        ButtonSegment(value: Granularity.day, label: Text('日')),
        ButtonSegment(value: Granularity.week, label: Text('周')),
        ButtonSegment(value: Granularity.month, label: Text('月')),
      ],
      selected: {current},
      onSelectionChanged: (v) =>
          ref.read(trendsGranularityProvider.notifier).set(v.first),
      emptySelectionAllowed: false,
    );
  }
}

class _OverviewGrid extends StatelessWidget {
  final Overview overview;

  const _OverviewGrid({required this.overview});

  @override
  Widget build(BuildContext context) {
    final isWide = MediaQuery.of(context).size.width >= 600;
    final scheme = Theme.of(context).colorScheme;

    final items = [
      _StatItem(
        icon: FluentIcons.organization_24_regular,
        color: scheme.primary,
        label: '租户',
        value: '${overview.totalTenants}',
      ),
      _StatItem(
        icon: FluentIcons.apps_24_regular,
        color: scheme.secondary,
        label: '应用',
        value: '${overview.totalApplications}',
      ),
      _StatItem(
        icon: FluentIcons.people_24_regular,
        color: scheme.tertiary,
        label: '用户',
        value: '${overview.totalApplicationUsers}',
      ),
      _StatItem(
        icon: FluentIcons.person_accounts_24_regular,
        color: scheme.error,
        label: '管理员',
        value: '${overview.totalAdministrators}',
      ),
      _StatItem(
        icon: FluentIcons.key_24_regular,
        color: scheme.outline,
        label: '活跃密钥',
        value: '${overview.totalActiveSecrets}',
      ),
    ];

    if (isWide) {
      return Column(
        spacing: 12,
        children: [
          Row(
            spacing: 12,
            children: items
                .sublist(0, 3)
                .map((e) => Expanded(child: e))
                .toList(),
          ),
          Row(
            spacing: 12,
            children: [
              ...items.sublist(3).map((e) => Expanded(child: e)),
              const Spacer(),
            ],
          ),
        ],
      );
    }

    return Column(
      spacing: 12,
      children: [
        Row(
          spacing: 12,
          children: items.sublist(0, 2).map((e) => Expanded(child: e)).toList(),
        ),
        Row(
          spacing: 12,
          children: items.sublist(2, 4).map((e) => Expanded(child: e)).toList(),
        ),
        items[4],
      ],
    );
  }
}

class _StatItem extends StatelessWidget {
  final IconData icon;
  final Color color;
  final String label;
  final String value;

  const _StatItem({
    required this.icon,
    required this.color,
    required this.label,
    required this.value,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card.filled(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(16, 16, 16, 0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              children: [
                Icon(icon, size: 20, color: color),
                const SizedBox(width: 8),
                Text(label, style: theme.textTheme.titleLarge),
              ],
            ),
            const SizedBox(height: 12),
            Text(value, style: theme.textTheme.displaySmall),
          ],
        ),
      ),
    );
  }
}

class _TrendsGrid extends StatelessWidget {
  final PlatformTrends trends;

  const _TrendsGrid({required this.trends});

  @override
  Widget build(BuildContext context) {
    final isWide = MediaQuery.of(context).size.width >= 600;
    final scheme = Theme.of(context).colorScheme;

    final charts = [
      _TrendChart(title: '租户', points: trends.tenants, color: scheme.primary),
      _TrendChart(
        title: '应用',
        points: trends.applications,
        color: scheme.secondary,
      ),
      _TrendChart(title: '用户', points: trends.users, color: scheme.tertiary),
      _TrendChart(
        title: '管理员',
        points: trends.administrators,
        color: scheme.error,
      ),
    ];

    if (isWide) {
      return Column(
        children: [
          Row(
            children: [
              Expanded(child: charts[0]),
              const SizedBox(width: 12),
              Expanded(child: charts[1]),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(child: charts[2]),
              const SizedBox(width: 12),
              Expanded(child: charts[3]),
            ],
          ),
        ],
      );
    }

    return Column(
      children: [
        charts[0],
        const SizedBox(height: 12),
        charts[1],
        const SizedBox(height: 12),
        charts[2],
        const SizedBox(height: 12),
        charts[3],
      ],
    );
  }
}

class _TrendChart extends StatelessWidget {
  final String title;
  final List<TrendDataPoint> points;
  final Color color;

  const _TrendChart({
    required this.title,
    required this.points,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final spots = points
        .asMap()
        .entries
        .map((e) => FlSpot(e.key.toDouble(), e.value.count.toDouble()))
        .toList();

    return Card.filled(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              children: [
                Container(
                  width: 12,
                  height: 12,
                  decoration: BoxDecoration(
                    color: color,
                    borderRadius: BorderRadius.circular(3),
                  ),
                ),
                const SizedBox(width: 8),
                Text(title, style: theme.textTheme.titleMedium),
              ],
            ),
            const SizedBox(height: 12),
            SizedBox(
              height: 160,
              child: spots.isEmpty
                  ? const Center(child: Text('暂无数据'))
                  : LineChart(
                      LineChartData(
                        gridData: FlGridData(
                          show: true,
                          // getDrawingHorizontalLine: (value) =>
                          //     FlLine(color: theme.focusColor, strokeWidth: 1),
                        ),
                        titlesData: FlTitlesData(
                          topTitles: AxisTitles(
                            sideTitles: SideTitles(showTitles: false),
                          ),
                          rightTitles: AxisTitles(
                            sideTitles: SideTitles(showTitles: false),
                          ),
                          leftTitles: AxisTitles(
                            sideTitles: SideTitles(showTitles: false),
                          ),
                        ),
                        borderData: FlBorderData(show: false),
                        lineBarsData: [
                          LineChartBarData(
                            spots: spots,
                            color: color,
                            barWidth: 2.5,
                            dotData: FlDotData(show: false),
                            belowBarData: BarAreaData(show: true),
                          ),
                        ],
                      ),
                    ),
            ),
          ],
        ),
      ),
    );
  }
}

class _SkeletonGrid extends StatelessWidget {
  final int count;

  const _SkeletonGrid({required this.count});

  @override
  Widget build(BuildContext context) {
    return GridView.extent(
      maxCrossAxisExtent: 200,
      crossAxisSpacing: 12,
      mainAxisSpacing: 12,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      childAspectRatio: 1.5,
      children: List.generate(
        count,
        (_) => Card(
          elevation: 0,
          child: const Center(
            child: SizedBox(
              width: 24,
              height: 24,
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
          ),
        ),
      ),
    );
  }
}

class _ErrorBox extends StatelessWidget {
  final String message;

  const _ErrorBox({required this.message});

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 0,
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
          ],
        ),
      ),
    );
  }
}
