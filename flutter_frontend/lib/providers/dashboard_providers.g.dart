// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'dashboard_providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$fetchOverviewHash() => r'd5cd82383c74f0d8fbe95f1eedcd8ca296d4e676';

/// See also [fetchOverview].
@ProviderFor(fetchOverview)
final fetchOverviewProvider = AutoDisposeFutureProvider<Overview>.internal(
  fetchOverview,
  name: r'fetchOverviewProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$fetchOverviewHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef FetchOverviewRef = AutoDisposeFutureProviderRef<Overview>;
String _$fetchPlatformTrendsHash() =>
    r'8dffe4ee0e72757dd86957d1f9ecc596d79fcf6d';

/// See also [fetchPlatformTrends].
@ProviderFor(fetchPlatformTrends)
final fetchPlatformTrendsProvider =
    AutoDisposeFutureProvider<PlatformTrends>.internal(
      fetchPlatformTrends,
      name: r'fetchPlatformTrendsProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$fetchPlatformTrendsHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef FetchPlatformTrendsRef = AutoDisposeFutureProviderRef<PlatformTrends>;
String _$trendsGranularityHash() => r'c063089ded71ba46075f282e051d455ef2548ccf';

/// See also [TrendsGranularity].
@ProviderFor(TrendsGranularity)
final trendsGranularityProvider =
    AutoDisposeNotifierProvider<TrendsGranularity, Granularity>.internal(
      TrendsGranularity.new,
      name: r'trendsGranularityProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$trendsGranularityHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

typedef _$TrendsGranularity = AutoDisposeNotifier<Granularity>;
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
