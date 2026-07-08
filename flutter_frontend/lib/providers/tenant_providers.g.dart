// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'tenant_providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$tenantListHash() => r'5ffc199a8a6abf5fb6ad6ef4c2f66ff6b4fb64e7';

/// See also [tenantList].
@ProviderFor(tenantList)
final tenantListProvider = FutureProvider<List<Tenant>>.internal(
  tenantList,
  name: r'tenantListProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$tenantListHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef TenantListRef = FutureProviderRef<List<Tenant>>;
String _$currentTenantHash() => r'284195bb96bf6d8b2b63f46445810621657f7e12';

/// See also [currentTenant].
@ProviderFor(currentTenant)
final currentTenantProvider = Provider<Tenant?>.internal(
  currentTenant,
  name: r'currentTenantProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$currentTenantHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef CurrentTenantRef = ProviderRef<Tenant?>;
String _$currentTenantIdHash() => r'9d76075c0d34a542c50aa46c61d3d227a3f3504e';

/// See also [CurrentTenantId].
@ProviderFor(CurrentTenantId)
final currentTenantIdProvider =
    NotifierProvider<CurrentTenantId, String?>.internal(
      CurrentTenantId.new,
      name: r'currentTenantIdProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$currentTenantIdHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

typedef _$CurrentTenantId = Notifier<String?>;
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
