import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'package:oceaniam_sdk/oceaniam_sdk.dart';
import 'oceaniam_client_provider.dart';

part 'dashboard_providers.g.dart';

/// Dashboard 筛选状态：聚合粒度 + 时间窗口（天数）。
@immutable
class DashboardFilter {
  final Granularity granularity;
  final int rangeDays;

  const DashboardFilter({required this.granularity, required this.rangeDays});

  DashboardFilter copyWith({Granularity? granularity, int? rangeDays}) {
    return DashboardFilter(
      granularity: granularity ?? this.granularity,
      rangeDays: rangeDays ?? this.rangeDays,
    );
  }

  static const initial = DashboardFilter(
    granularity: Granularity.day,
    rangeDays: 30,
  );
}

/// 当前 Dashboard 筛选状态。
@riverpod
class DashboardFilterController extends _$DashboardFilterController {
  @override
  DashboardFilter build() => DashboardFilter.initial;

  void setGranularity(Granularity g) {
    if (state.granularity == g) return;
    state = state.copyWith(granularity: g);
  }

  void setRangeDays(int days) {
    if (days < 1) return;
    state = state.copyWith(rangeDays: days);
  }
}

/// 平台概览（不受筛选影响）。
@riverpod
Future<Overview> dashboardOverview(Ref ref) {
  final client = ref.watch(oceanIAMClientProvider);
  return client.getOverview();
}

/// 平台趋势（随筛选变化）。
@riverpod
Future<PlatformTrends> dashboardTrends(Ref ref) {
  final filter = ref.watch(dashboardFilterControllerProvider);
  final client = ref.watch(oceanIAMClientProvider);
  return client.getPlatformTrends(
    granularity: filter.granularity.name,
    range: filter.rangeDays,
  );
}
