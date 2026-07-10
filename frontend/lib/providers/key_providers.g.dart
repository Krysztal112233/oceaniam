// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'key_providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$tenantKeysHash() => r'500facecffeaff997c2f25d6f7aeb07f910065c6';

/// Copied from Dart SDK
class _SystemHash {
  _SystemHash._();

  static int combine(int hash, int value) {
    // ignore: parameter_assignments
    hash = 0x1fffffff & (hash + value);
    // ignore: parameter_assignments
    hash = 0x1fffffff & (hash + ((0x0007ffff & hash) << 10));
    return hash ^ (hash >> 6);
  }

  static int finish(int hash) {
    // ignore: parameter_assignments
    hash = 0x1fffffff & (hash + ((0x03ffffff & hash) << 3));
    // ignore: parameter_assignments
    hash = hash ^ (hash >> 11);
    return 0x1fffffff & (hash + ((0x00003fff & hash) << 15));
  }
}

/// See also [tenantKeys].
@ProviderFor(tenantKeys)
const tenantKeysProvider = TenantKeysFamily();

/// See also [tenantKeys].
class TenantKeysFamily extends Family<AsyncValue<List<ApplicationKey>>> {
  /// See also [tenantKeys].
  const TenantKeysFamily();

  /// See also [tenantKeys].
  TenantKeysProvider call(String tenantId) {
    return TenantKeysProvider(tenantId);
  }

  @override
  TenantKeysProvider getProviderOverride(
    covariant TenantKeysProvider provider,
  ) {
    return call(provider.tenantId);
  }

  static const Iterable<ProviderOrFamily>? _dependencies = null;

  @override
  Iterable<ProviderOrFamily>? get dependencies => _dependencies;

  static const Iterable<ProviderOrFamily>? _allTransitiveDependencies = null;

  @override
  Iterable<ProviderOrFamily>? get allTransitiveDependencies =>
      _allTransitiveDependencies;

  @override
  String? get name => r'tenantKeysProvider';
}

/// See also [tenantKeys].
class TenantKeysProvider
    extends AutoDisposeFutureProvider<List<ApplicationKey>> {
  /// See also [tenantKeys].
  TenantKeysProvider(String tenantId)
    : this._internal(
        (ref) => tenantKeys(ref as TenantKeysRef, tenantId),
        from: tenantKeysProvider,
        name: r'tenantKeysProvider',
        debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
            ? null
            : _$tenantKeysHash,
        dependencies: TenantKeysFamily._dependencies,
        allTransitiveDependencies: TenantKeysFamily._allTransitiveDependencies,
        tenantId: tenantId,
      );

  TenantKeysProvider._internal(
    super._createNotifier, {
    required super.name,
    required super.dependencies,
    required super.allTransitiveDependencies,
    required super.debugGetCreateSourceHash,
    required super.from,
    required this.tenantId,
  }) : super.internal();

  final String tenantId;

  @override
  Override overrideWith(
    FutureOr<List<ApplicationKey>> Function(TenantKeysRef provider) create,
  ) {
    return ProviderOverride(
      origin: this,
      override: TenantKeysProvider._internal(
        (ref) => create(ref as TenantKeysRef),
        from: from,
        name: null,
        dependencies: null,
        allTransitiveDependencies: null,
        debugGetCreateSourceHash: null,
        tenantId: tenantId,
      ),
    );
  }

  @override
  AutoDisposeFutureProviderElement<List<ApplicationKey>> createElement() {
    return _TenantKeysProviderElement(this);
  }

  @override
  bool operator ==(Object other) {
    return other is TenantKeysProvider && other.tenantId == tenantId;
  }

  @override
  int get hashCode {
    var hash = _SystemHash.combine(0, runtimeType.hashCode);
    hash = _SystemHash.combine(hash, tenantId.hashCode);

    return _SystemHash.finish(hash);
  }
}

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
mixin TenantKeysRef on AutoDisposeFutureProviderRef<List<ApplicationKey>> {
  /// The parameter `tenantId` of this provider.
  String get tenantId;
}

class _TenantKeysProviderElement
    extends AutoDisposeFutureProviderElement<List<ApplicationKey>>
    with TenantKeysRef {
  _TenantKeysProviderElement(super.provider);

  @override
  String get tenantId => (origin as TenantKeysProvider).tenantId;
}

// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
