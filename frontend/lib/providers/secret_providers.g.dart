// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'secret_providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$secretsPageHash() => r'57d147a6e1cc9ca1170ca6e43f97473db802d010';

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

/// See also [secretsPage].
@ProviderFor(secretsPage)
const secretsPageProvider = SecretsPageFamily();

/// See also [secretsPage].
class SecretsPageFamily extends Family<AsyncValue<PagedResponse<Secret>>> {
  /// See also [secretsPage].
  const SecretsPageFamily();

  /// See also [secretsPage].
  SecretsPageProvider call(int page) {
    return SecretsPageProvider(page);
  }

  @override
  SecretsPageProvider getProviderOverride(
    covariant SecretsPageProvider provider,
  ) {
    return call(provider.page);
  }

  static const Iterable<ProviderOrFamily>? _dependencies = null;

  @override
  Iterable<ProviderOrFamily>? get dependencies => _dependencies;

  static const Iterable<ProviderOrFamily>? _allTransitiveDependencies = null;

  @override
  Iterable<ProviderOrFamily>? get allTransitiveDependencies =>
      _allTransitiveDependencies;

  @override
  String? get name => r'secretsPageProvider';
}

/// See also [secretsPage].
class SecretsPageProvider
    extends AutoDisposeFutureProvider<PagedResponse<Secret>> {
  /// See also [secretsPage].
  SecretsPageProvider(int page)
    : this._internal(
        (ref) => secretsPage(ref as SecretsPageRef, page),
        from: secretsPageProvider,
        name: r'secretsPageProvider',
        debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
            ? null
            : _$secretsPageHash,
        dependencies: SecretsPageFamily._dependencies,
        allTransitiveDependencies: SecretsPageFamily._allTransitiveDependencies,
        page: page,
      );

  SecretsPageProvider._internal(
    super._createNotifier, {
    required super.name,
    required super.dependencies,
    required super.allTransitiveDependencies,
    required super.debugGetCreateSourceHash,
    required super.from,
    required this.page,
  }) : super.internal();

  final int page;

  @override
  Override overrideWith(
    FutureOr<PagedResponse<Secret>> Function(SecretsPageRef provider) create,
  ) {
    return ProviderOverride(
      origin: this,
      override: SecretsPageProvider._internal(
        (ref) => create(ref as SecretsPageRef),
        from: from,
        name: null,
        dependencies: null,
        allTransitiveDependencies: null,
        debugGetCreateSourceHash: null,
        page: page,
      ),
    );
  }

  @override
  AutoDisposeFutureProviderElement<PagedResponse<Secret>> createElement() {
    return _SecretsPageProviderElement(this);
  }

  @override
  bool operator ==(Object other) {
    return other is SecretsPageProvider && other.page == page;
  }

  @override
  int get hashCode {
    var hash = _SystemHash.combine(0, runtimeType.hashCode);
    hash = _SystemHash.combine(hash, page.hashCode);

    return _SystemHash.finish(hash);
  }
}

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
mixin SecretsPageRef on AutoDisposeFutureProviderRef<PagedResponse<Secret>> {
  /// The parameter `page` of this provider.
  int get page;
}

class _SecretsPageProviderElement
    extends AutoDisposeFutureProviderElement<PagedResponse<Secret>>
    with SecretsPageRef {
  _SecretsPageProviderElement(super.provider);

  @override
  int get page => (origin as SecretsPageProvider).page;
}

// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
