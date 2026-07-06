import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../providers/dashboard_providers.dart';

/// Dashboard 筛选条：聚合粒度（按日/按周/按月）+ 时间窗口（日期范围选择器）。
///
/// 粒度通过 [SegmentedButton] 切换，时间窗口通过 [showDateRangePicker]
/// 选择起止日期，差值天数作为 `range` 传给后端。默认：day + 30 天。
class TrendFilterBar extends ConsumerWidget {
  const TrendFilterBar({super.key});

  static const _granularityLabels = {
    Granularity.day: 'Day',
    Granularity.week: 'Week',
    Granularity.month: 'Month',
  };

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final filter = ref.watch(dashboardFilterControllerProvider);
    final controller = ref.read(dashboardFilterControllerProvider.notifier);

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Wrap(
        spacing: 12,
        crossAxisAlignment: WrapCrossAlignment.center,
        children: [
          SegmentedButton<Granularity>(
            segments: Granularity.values
                .map(
                  (g) => ButtonSegment(
                    value: g,
                    label: Text(_granularityLabels[g] ?? g.name),
                  ),
                )
                .toList(),
            selected: {filter.granularity},
            onSelectionChanged: (s) => controller.setGranularity(s.first),
            style: const ButtonStyle(
              visualDensity: VisualDensity(horizontal: -3, vertical: -2),
            ),
          ),
          ActionChip(
            avatar: const Icon(Icons.date_range, size: 18),
            label: Text(_rangeLabel(filter.rangeDays)),
            onPressed: () => _pickDateRange(context, ref, filter.rangeDays),
          ),
        ],
      ),
    );
  }

  Future<void> _pickDateRange(
    BuildContext context,
    WidgetRef ref,
    int currentDays,
  ) async {
    final now = DateTime.now();
    final today = DateTime(now.year, now.month, now.day);
    final initialStart = today.subtract(Duration(days: currentDays - 1));
    final picked = await showDateRangePicker(
      context: context,
      firstDate: DateTime(today.year - 5),
      lastDate: today,
      initialDateRange: DateTimeRange(start: initialStart, end: today),
      helpText: 'Select trend range',
    );
    if (picked == null) return;
    final days = picked.end.difference(picked.start).inDays + 1;
    ref.read(dashboardFilterControllerProvider.notifier).setRangeDays(days);
  }

  String _rangeLabel(int days) {
    if (days == 30) return 'Last 30 days';
    if (days == 7) return 'Last 7 days';
    if (days == 90) return 'Last 90 days';
    return '$days days';
  }
}
