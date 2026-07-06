// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'dashboard_providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$dashboardOverviewHash() => r'5e17e0c9bb2cd2c920ba2157a9bd12236f649721';

/// 平台概览（不受筛选影响）。
///
/// Copied from [dashboardOverview].
@ProviderFor(dashboardOverview)
final dashboardOverviewProvider = AutoDisposeFutureProvider<Overview>.internal(
  dashboardOverview,
  name: r'dashboardOverviewProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$dashboardOverviewHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef DashboardOverviewRef = AutoDisposeFutureProviderRef<Overview>;
String _$dashboardTrendsHash() => r'51fafb714ce8f450b2b3617533f8afddc8a0f8ed';

/// 平台趋势（随筛选变化）。
///
/// Copied from [dashboardTrends].
@ProviderFor(dashboardTrends)
final dashboardTrendsProvider =
    AutoDisposeFutureProvider<PlatformTrends>.internal(
      dashboardTrends,
      name: r'dashboardTrendsProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$dashboardTrendsHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef DashboardTrendsRef = AutoDisposeFutureProviderRef<PlatformTrends>;
String _$dashboardFilterControllerHash() =>
    r'02e373ae805116bb3b7625630570e619cd7a9df7';

/// 当前 Dashboard 筛选状态。
///
/// Copied from [DashboardFilterController].
@ProviderFor(DashboardFilterController)
final dashboardFilterControllerProvider =
    AutoDisposeNotifierProvider<
      DashboardFilterController,
      DashboardFilter
    >.internal(
      DashboardFilterController.new,
      name: r'dashboardFilterControllerProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$dashboardFilterControllerHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

typedef _$DashboardFilterController = AutoDisposeNotifier<DashboardFilter>;
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
