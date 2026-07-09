// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'application_providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$applicationListHash() => r'd63fd521a07b8c8554774747c5f11ad4088dd06e';

/// See also [applicationList].
@ProviderFor(applicationList)
final applicationListProvider = FutureProvider<List<Application>>.internal(
  applicationList,
  name: r'applicationListProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$applicationListHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef ApplicationListRef = FutureProviderRef<List<Application>>;
String _$applicationUsersHash() => r'056e7fb3c2c936e65714492e2b87663b4e00544d';

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

/// See also [applicationUsers].
@ProviderFor(applicationUsers)
const applicationUsersProvider = ApplicationUsersFamily();

/// See also [applicationUsers].
class ApplicationUsersFamily extends Family<AsyncValue<List<ApplicationUser>>> {
  /// See also [applicationUsers].
  const ApplicationUsersFamily();

  /// See also [applicationUsers].
  ApplicationUsersProvider call(String tenantId, String applicationId) {
    return ApplicationUsersProvider(tenantId, applicationId);
  }

  @override
  ApplicationUsersProvider getProviderOverride(
    covariant ApplicationUsersProvider provider,
  ) {
    return call(provider.tenantId, provider.applicationId);
  }

  static const Iterable<ProviderOrFamily>? _dependencies = null;

  @override
  Iterable<ProviderOrFamily>? get dependencies => _dependencies;

  static const Iterable<ProviderOrFamily>? _allTransitiveDependencies = null;

  @override
  Iterable<ProviderOrFamily>? get allTransitiveDependencies =>
      _allTransitiveDependencies;

  @override
  String? get name => r'applicationUsersProvider';
}

/// See also [applicationUsers].
class ApplicationUsersProvider
    extends AutoDisposeFutureProvider<List<ApplicationUser>> {
  /// See also [applicationUsers].
  ApplicationUsersProvider(String tenantId, String applicationId)
    : this._internal(
        (ref) => applicationUsers(
          ref as ApplicationUsersRef,
          tenantId,
          applicationId,
        ),
        from: applicationUsersProvider,
        name: r'applicationUsersProvider',
        debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
            ? null
            : _$applicationUsersHash,
        dependencies: ApplicationUsersFamily._dependencies,
        allTransitiveDependencies:
            ApplicationUsersFamily._allTransitiveDependencies,
        tenantId: tenantId,
        applicationId: applicationId,
      );

  ApplicationUsersProvider._internal(
    super._createNotifier, {
    required super.name,
    required super.dependencies,
    required super.allTransitiveDependencies,
    required super.debugGetCreateSourceHash,
    required super.from,
    required this.tenantId,
    required this.applicationId,
  }) : super.internal();

  final String tenantId;
  final String applicationId;

  @override
  Override overrideWith(
    FutureOr<List<ApplicationUser>> Function(ApplicationUsersRef provider)
    create,
  ) {
    return ProviderOverride(
      origin: this,
      override: ApplicationUsersProvider._internal(
        (ref) => create(ref as ApplicationUsersRef),
        from: from,
        name: null,
        dependencies: null,
        allTransitiveDependencies: null,
        debugGetCreateSourceHash: null,
        tenantId: tenantId,
        applicationId: applicationId,
      ),
    );
  }

  @override
  AutoDisposeFutureProviderElement<List<ApplicationUser>> createElement() {
    return _ApplicationUsersProviderElement(this);
  }

  @override
  bool operator ==(Object other) {
    return other is ApplicationUsersProvider &&
        other.tenantId == tenantId &&
        other.applicationId == applicationId;
  }

  @override
  int get hashCode {
    var hash = _SystemHash.combine(0, runtimeType.hashCode);
    hash = _SystemHash.combine(hash, tenantId.hashCode);
    hash = _SystemHash.combine(hash, applicationId.hashCode);

    return _SystemHash.finish(hash);
  }
}

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
mixin ApplicationUsersRef
    on AutoDisposeFutureProviderRef<List<ApplicationUser>> {
  /// The parameter `tenantId` of this provider.
  String get tenantId;

  /// The parameter `applicationId` of this provider.
  String get applicationId;
}

class _ApplicationUsersProviderElement
    extends AutoDisposeFutureProviderElement<List<ApplicationUser>>
    with ApplicationUsersRef {
  _ApplicationUsersProviderElement(super.provider);

  @override
  String get tenantId => (origin as ApplicationUsersProvider).tenantId;
  @override
  String get applicationId =>
      (origin as ApplicationUsersProvider).applicationId;
}

// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
