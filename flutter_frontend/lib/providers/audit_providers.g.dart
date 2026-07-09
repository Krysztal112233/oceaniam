// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'audit_providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$auditsPageHash() => r'7d5d14605c2eb73027aa64f5cb4fd1ecf3357b0b';

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

/// See also [auditsPage].
@ProviderFor(auditsPage)
const auditsPageProvider = AuditsPageFamily();

/// See also [auditsPage].
class AuditsPageFamily extends Family<AsyncValue<PagedResponse<AuditLog>>> {
  /// See also [auditsPage].
  const AuditsPageFamily();

  /// See also [auditsPage].
  AuditsPageProvider call(int page, String? auditType) {
    return AuditsPageProvider(page, auditType);
  }

  @override
  AuditsPageProvider getProviderOverride(
    covariant AuditsPageProvider provider,
  ) {
    return call(provider.page, provider.auditType);
  }

  static const Iterable<ProviderOrFamily>? _dependencies = null;

  @override
  Iterable<ProviderOrFamily>? get dependencies => _dependencies;

  static const Iterable<ProviderOrFamily>? _allTransitiveDependencies = null;

  @override
  Iterable<ProviderOrFamily>? get allTransitiveDependencies =>
      _allTransitiveDependencies;

  @override
  String? get name => r'auditsPageProvider';
}

/// See also [auditsPage].
class AuditsPageProvider
    extends AutoDisposeFutureProvider<PagedResponse<AuditLog>> {
  /// See also [auditsPage].
  AuditsPageProvider(int page, String? auditType)
    : this._internal(
        (ref) => auditsPage(ref as AuditsPageRef, page, auditType),
        from: auditsPageProvider,
        name: r'auditsPageProvider',
        debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
            ? null
            : _$auditsPageHash,
        dependencies: AuditsPageFamily._dependencies,
        allTransitiveDependencies: AuditsPageFamily._allTransitiveDependencies,
        page: page,
        auditType: auditType,
      );

  AuditsPageProvider._internal(
    super._createNotifier, {
    required super.name,
    required super.dependencies,
    required super.allTransitiveDependencies,
    required super.debugGetCreateSourceHash,
    required super.from,
    required this.page,
    required this.auditType,
  }) : super.internal();

  final int page;
  final String? auditType;

  @override
  Override overrideWith(
    FutureOr<PagedResponse<AuditLog>> Function(AuditsPageRef provider) create,
  ) {
    return ProviderOverride(
      origin: this,
      override: AuditsPageProvider._internal(
        (ref) => create(ref as AuditsPageRef),
        from: from,
        name: null,
        dependencies: null,
        allTransitiveDependencies: null,
        debugGetCreateSourceHash: null,
        page: page,
        auditType: auditType,
      ),
    );
  }

  @override
  AutoDisposeFutureProviderElement<PagedResponse<AuditLog>> createElement() {
    return _AuditsPageProviderElement(this);
  }

  @override
  bool operator ==(Object other) {
    return other is AuditsPageProvider &&
        other.page == page &&
        other.auditType == auditType;
  }

  @override
  int get hashCode {
    var hash = _SystemHash.combine(0, runtimeType.hashCode);
    hash = _SystemHash.combine(hash, page.hashCode);
    hash = _SystemHash.combine(hash, auditType.hashCode);

    return _SystemHash.finish(hash);
  }
}

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
mixin AuditsPageRef on AutoDisposeFutureProviderRef<PagedResponse<AuditLog>> {
  /// The parameter `page` of this provider.
  int get page;

  /// The parameter `auditType` of this provider.
  String? get auditType;
}

class _AuditsPageProviderElement
    extends AutoDisposeFutureProviderElement<PagedResponse<AuditLog>>
    with AuditsPageRef {
  _AuditsPageProviderElement(super.provider);

  @override
  int get page => (origin as AuditsPageProvider).page;
  @override
  String? get auditType => (origin as AuditsPageProvider).auditType;
}

// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
