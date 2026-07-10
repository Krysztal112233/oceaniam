// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'oceaniam_client_provider.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$oceanIAMClientHash() => r'8a229d71a1bd020fdb1189114b37019d071c455b';

/// 单例 [OceanIAMClient] provider。
///
/// 启动时从 [SharedPreferences] 读取已保存的 JWT 并注入到 client，
/// 使应用刷新后仍处于已登录状态。token 的写入由 [AuthController] 负责。
///
/// Copied from [oceanIAMClient].
@ProviderFor(oceanIAMClient)
final oceanIAMClientProvider = Provider<OceanIAMClient>.internal(
  oceanIAMClient,
  name: r'oceanIAMClientProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$oceanIAMClientHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef OceanIAMClientRef = ProviderRef<OceanIAMClient>;
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
