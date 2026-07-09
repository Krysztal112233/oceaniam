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
String _$applicationUsersPageHash() =>
    r'0b28563849c70c4d06f735f60698c44db9116cb7';

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

/// Lists users, or searches when [searchQuery] is non-empty.
///
/// [searchField] selects which `by_*` query param is sent. Empty [searchQuery]
/// falls back to the paginated list endpoint.
///
/// Copied from [applicationUsersPage].
@ProviderFor(applicationUsersPage)
const applicationUsersPageProvider = ApplicationUsersPageFamily();

/// Lists users, or searches when [searchQuery] is non-empty.
///
/// [searchField] selects which `by_*` query param is sent. Empty [searchQuery]
/// falls back to the paginated list endpoint.
///
/// Copied from [applicationUsersPage].
class ApplicationUsersPageFamily
    extends Family<AsyncValue<PagedResponse<ApplicationUser>>> {
  /// Lists users, or searches when [searchQuery] is non-empty.
  ///
  /// [searchField] selects which `by_*` query param is sent. Empty [searchQuery]
  /// falls back to the paginated list endpoint.
  ///
  /// Copied from [applicationUsersPage].
  const ApplicationUsersPageFamily();

  /// Lists users, or searches when [searchQuery] is non-empty.
  ///
  /// [searchField] selects which `by_*` query param is sent. Empty [searchQuery]
  /// falls back to the paginated list endpoint.
  ///
  /// Copied from [applicationUsersPage].
  ApplicationUsersPageProvider call(
    String tenantId,
    String applicationId,
    int page,
    ApplicationUserSearchField searchField,
    String searchQuery,
  ) {
    return ApplicationUsersPageProvider(
      tenantId,
      applicationId,
      page,
      searchField,
      searchQuery,
    );
  }

  @override
  ApplicationUsersPageProvider getProviderOverride(
    covariant ApplicationUsersPageProvider provider,
  ) {
    return call(
      provider.tenantId,
      provider.applicationId,
      provider.page,
      provider.searchField,
      provider.searchQuery,
    );
  }

  static const Iterable<ProviderOrFamily>? _dependencies = null;

  @override
  Iterable<ProviderOrFamily>? get dependencies => _dependencies;

  static const Iterable<ProviderOrFamily>? _allTransitiveDependencies = null;

  @override
  Iterable<ProviderOrFamily>? get allTransitiveDependencies =>
      _allTransitiveDependencies;

  @override
  String? get name => r'applicationUsersPageProvider';
}

/// Lists users, or searches when [searchQuery] is non-empty.
///
/// [searchField] selects which `by_*` query param is sent. Empty [searchQuery]
/// falls back to the paginated list endpoint.
///
/// Copied from [applicationUsersPage].
class ApplicationUsersPageProvider
    extends AutoDisposeFutureProvider<PagedResponse<ApplicationUser>> {
  /// Lists users, or searches when [searchQuery] is non-empty.
  ///
  /// [searchField] selects which `by_*` query param is sent. Empty [searchQuery]
  /// falls back to the paginated list endpoint.
  ///
  /// Copied from [applicationUsersPage].
  ApplicationUsersPageProvider(
    String tenantId,
    String applicationId,
    int page,
    ApplicationUserSearchField searchField,
    String searchQuery,
  ) : this._internal(
        (ref) => applicationUsersPage(
          ref as ApplicationUsersPageRef,
          tenantId,
          applicationId,
          page,
          searchField,
          searchQuery,
        ),
        from: applicationUsersPageProvider,
        name: r'applicationUsersPageProvider',
        debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
            ? null
            : _$applicationUsersPageHash,
        dependencies: ApplicationUsersPageFamily._dependencies,
        allTransitiveDependencies:
            ApplicationUsersPageFamily._allTransitiveDependencies,
        tenantId: tenantId,
        applicationId: applicationId,
        page: page,
        searchField: searchField,
        searchQuery: searchQuery,
      );

  ApplicationUsersPageProvider._internal(
    super._createNotifier, {
    required super.name,
    required super.dependencies,
    required super.allTransitiveDependencies,
    required super.debugGetCreateSourceHash,
    required super.from,
    required this.tenantId,
    required this.applicationId,
    required this.page,
    required this.searchField,
    required this.searchQuery,
  }) : super.internal();

  final String tenantId;
  final String applicationId;
  final int page;
  final ApplicationUserSearchField searchField;
  final String searchQuery;

  @override
  Override overrideWith(
    FutureOr<PagedResponse<ApplicationUser>> Function(
      ApplicationUsersPageRef provider,
    )
    create,
  ) {
    return ProviderOverride(
      origin: this,
      override: ApplicationUsersPageProvider._internal(
        (ref) => create(ref as ApplicationUsersPageRef),
        from: from,
        name: null,
        dependencies: null,
        allTransitiveDependencies: null,
        debugGetCreateSourceHash: null,
        tenantId: tenantId,
        applicationId: applicationId,
        page: page,
        searchField: searchField,
        searchQuery: searchQuery,
      ),
    );
  }

  @override
  AutoDisposeFutureProviderElement<PagedResponse<ApplicationUser>>
  createElement() {
    return _ApplicationUsersPageProviderElement(this);
  }

  @override
  bool operator ==(Object other) {
    return other is ApplicationUsersPageProvider &&
        other.tenantId == tenantId &&
        other.applicationId == applicationId &&
        other.page == page &&
        other.searchField == searchField &&
        other.searchQuery == searchQuery;
  }

  @override
  int get hashCode {
    var hash = _SystemHash.combine(0, runtimeType.hashCode);
    hash = _SystemHash.combine(hash, tenantId.hashCode);
    hash = _SystemHash.combine(hash, applicationId.hashCode);
    hash = _SystemHash.combine(hash, page.hashCode);
    hash = _SystemHash.combine(hash, searchField.hashCode);
    hash = _SystemHash.combine(hash, searchQuery.hashCode);

    return _SystemHash.finish(hash);
  }
}

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
mixin ApplicationUsersPageRef
    on AutoDisposeFutureProviderRef<PagedResponse<ApplicationUser>> {
  /// The parameter `tenantId` of this provider.
  String get tenantId;

  /// The parameter `applicationId` of this provider.
  String get applicationId;

  /// The parameter `page` of this provider.
  int get page;

  /// The parameter `searchField` of this provider.
  ApplicationUserSearchField get searchField;

  /// The parameter `searchQuery` of this provider.
  String get searchQuery;
}

class _ApplicationUsersPageProviderElement
    extends AutoDisposeFutureProviderElement<PagedResponse<ApplicationUser>>
    with ApplicationUsersPageRef {
  _ApplicationUsersPageProviderElement(super.provider);

  @override
  String get tenantId => (origin as ApplicationUsersPageProvider).tenantId;
  @override
  String get applicationId =>
      (origin as ApplicationUsersPageProvider).applicationId;
  @override
  int get page => (origin as ApplicationUsersPageProvider).page;
  @override
  ApplicationUserSearchField get searchField =>
      (origin as ApplicationUsersPageProvider).searchField;
  @override
  String get searchQuery =>
      (origin as ApplicationUsersPageProvider).searchQuery;
}

// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
