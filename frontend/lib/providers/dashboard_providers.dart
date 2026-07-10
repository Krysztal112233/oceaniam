import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import 'oceaniam_client_provider.dart';

part 'dashboard_providers.g.dart';

DateTime _truncate(DateTime dt, Granularity g) {
  switch (g) {
    case Granularity.day:
      return DateTime(dt.year, dt.month, dt.day, dt.hour);
    case Granularity.week:
    case Granularity.month:
      return DateTime(dt.year, dt.month, dt.day);
  }
}

List<TrendDataPoint> _padTrendData(
  List<TrendDataPoint> points,
  Granularity granularity,
  DateTime now,
) {
  final int count;
  final Duration step;
  switch (granularity) {
    case Granularity.day:
      count = 24;
      step = const Duration(hours: 1);
    case Granularity.week:
      count = 7;
      step = const Duration(days: 1);
    case Granularity.month:
      count = 30;
      step = const Duration(days: 1);
  }

  final truncatedNow = _truncate(now, granularity);
  final buckets = List.generate(count, (i) {
    return truncatedNow.subtract(step * (count - 1 - i));
  });

  final pointMap = <DateTime, int>{
    for (final p in points) _truncate(p.bucket, granularity): p.count,
  };

  return buckets.map((b) {
    return TrendDataPoint(bucket: b, count: pointMap[b] ?? 0);
  }).toList();
}

@riverpod
Future<Overview> fetchOverview(Ref ref) async {
  final client = ref.watch(oceanIAMClientProvider);
  return client.getOverview();
}

@riverpod
class TrendsGranularity extends _$TrendsGranularity {
  @override
  Granularity build() => Granularity.day;

  void set(Granularity value) => state = value;
}

@Riverpod(keepAlive: true)
Future<PlatformTrends> fetchPlatformTrends(Ref ref) async {
  final client = ref.watch(oceanIAMClientProvider);
  final granularity = ref.watch(trendsGranularityProvider);
  final raw = await client.getPlatformTrends(
    granularity: granularity.name,
    range: 30,
  );
  final now = DateTime.now();
  return PlatformTrends(
    granularity: raw.granularity,
    range: raw.range,
    tenants: _padTrendData(raw.tenants, granularity, now),
    applications: _padTrendData(raw.applications, granularity, now),
    users: _padTrendData(raw.users, granularity, now),
    administrators: _padTrendData(raw.administrators, granularity, now),
  );
}
