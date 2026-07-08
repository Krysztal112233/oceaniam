import 'package:freezed_annotation/freezed_annotation.dart';

part 'statistics.freezed.dart';
part 'statistics.g.dart';

@freezed
class Overview with _$Overview {
  const factory Overview({
    @JsonKey(name: 'total_tenants') required int totalTenants,
    @JsonKey(name: 'total_applications') required int totalApplications,
    @JsonKey(name: 'total_administrators') required int totalAdministrators,
    @JsonKey(name: 'total_application_users')
    required int totalApplicationUsers,
    @JsonKey(name: 'total_active_secrets') required int totalActiveSecrets,
  }) = _Overview;

  factory Overview.fromJson(Map<String, dynamic> json) =>
      _$OverviewFromJson(json);
}

@freezed
class TrendDataPoint with _$TrendDataPoint {
  const factory TrendDataPoint({required DateTime bucket, required int count}) =
      _TrendDataPoint;

  factory TrendDataPoint.fromJson(Map<String, dynamic> json) =>
      _$TrendDataPointFromJson(json);
}

@freezed
class PlatformTrends with _$PlatformTrends {
  const factory PlatformTrends({
    required String granularity,
    required int range,
    required List<TrendDataPoint> tenants,
    required List<TrendDataPoint> applications,
    required List<TrendDataPoint> users,
    required List<TrendDataPoint> administrators,
  }) = _PlatformTrends;

  factory PlatformTrends.fromJson(Map<String, dynamic> json) =>
      _$PlatformTrendsFromJson(json);
}

@freezed
class ApplicationStatistics with _$ApplicationStatistics {
  const factory ApplicationStatistics({
    @JsonKey(name: 'total_users') required int totalUsers,
    @JsonKey(name: 'active_users') required int activeUsers,
  }) = _ApplicationStatistics;

  factory ApplicationStatistics.fromJson(Map<String, dynamic> json) =>
      _$ApplicationStatisticsFromJson(json);
}

@freezed
class ApplicationTrends with _$ApplicationTrends {
  const factory ApplicationTrends({
    required String granularity,
    required int range,
    @JsonKey(name: 'new_users') required List<TrendDataPoint> newUsers,
  }) = _ApplicationTrends;

  factory ApplicationTrends.fromJson(Map<String, dynamic> json) =>
      _$ApplicationTrendsFromJson(json);
}

enum Granularity {
  @JsonValue('day')
  day,
  @JsonValue('week')
  week,
  @JsonValue('month')
  month,
}
