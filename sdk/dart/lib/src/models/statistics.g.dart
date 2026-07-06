// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'statistics.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$OverviewImpl _$$OverviewImplFromJson(Map<String, dynamic> json) =>
    _$OverviewImpl(
      totalTenants: (json['total_tenants'] as num).toInt(),
      totalApplications: (json['total_applications'] as num).toInt(),
      totalAdministrators: (json['total_administrators'] as num).toInt(),
      totalApplicationUsers: (json['total_application_users'] as num).toInt(),
      totalActiveSecrets: (json['total_active_secrets'] as num).toInt(),
    );

Map<String, dynamic> _$$OverviewImplToJson(_$OverviewImpl instance) =>
    <String, dynamic>{
      'total_tenants': instance.totalTenants,
      'total_applications': instance.totalApplications,
      'total_administrators': instance.totalAdministrators,
      'total_application_users': instance.totalApplicationUsers,
      'total_active_secrets': instance.totalActiveSecrets,
    };

_$TrendDataPointImpl _$$TrendDataPointImplFromJson(Map<String, dynamic> json) =>
    _$TrendDataPointImpl(
      bucket: DateTime.parse(json['bucket'] as String),
      count: (json['count'] as num).toInt(),
    );

Map<String, dynamic> _$$TrendDataPointImplToJson(
        _$TrendDataPointImpl instance) =>
    <String, dynamic>{
      'bucket': instance.bucket.toIso8601String(),
      'count': instance.count,
    };

_$PlatformTrendsImpl _$$PlatformTrendsImplFromJson(Map<String, dynamic> json) =>
    _$PlatformTrendsImpl(
      granularity: json['granularity'] as String,
      range: (json['range'] as num).toInt(),
      tenants: (json['tenants'] as List<dynamic>)
          .map((e) => TrendDataPoint.fromJson(e as Map<String, dynamic>))
          .toList(),
      applications: (json['applications'] as List<dynamic>)
          .map((e) => TrendDataPoint.fromJson(e as Map<String, dynamic>))
          .toList(),
      users: (json['users'] as List<dynamic>)
          .map((e) => TrendDataPoint.fromJson(e as Map<String, dynamic>))
          .toList(),
      administrators: (json['administrators'] as List<dynamic>)
          .map((e) => TrendDataPoint.fromJson(e as Map<String, dynamic>))
          .toList(),
    );

Map<String, dynamic> _$$PlatformTrendsImplToJson(
        _$PlatformTrendsImpl instance) =>
    <String, dynamic>{
      'granularity': instance.granularity,
      'range': instance.range,
      'tenants': instance.tenants,
      'applications': instance.applications,
      'users': instance.users,
      'administrators': instance.administrators,
    };

_$ApplicationStatisticsImpl _$$ApplicationStatisticsImplFromJson(
        Map<String, dynamic> json) =>
    _$ApplicationStatisticsImpl(
      totalUsers: (json['total_users'] as num).toInt(),
      activeUsers: (json['active_users'] as num).toInt(),
    );

Map<String, dynamic> _$$ApplicationStatisticsImplToJson(
        _$ApplicationStatisticsImpl instance) =>
    <String, dynamic>{
      'total_users': instance.totalUsers,
      'active_users': instance.activeUsers,
    };

_$ApplicationTrendsImpl _$$ApplicationTrendsImplFromJson(
        Map<String, dynamic> json) =>
    _$ApplicationTrendsImpl(
      granularity: json['granularity'] as String,
      range: (json['range'] as num).toInt(),
      newUsers: (json['new_users'] as List<dynamic>)
          .map((e) => TrendDataPoint.fromJson(e as Map<String, dynamic>))
          .toList(),
    );

Map<String, dynamic> _$$ApplicationTrendsImplToJson(
        _$ApplicationTrendsImpl instance) =>
    <String, dynamic>{
      'granularity': instance.granularity,
      'range': instance.range,
      'new_users': instance.newUsers,
    };
