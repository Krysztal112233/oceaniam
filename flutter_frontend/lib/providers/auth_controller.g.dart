// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'auth_controller.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$authControllerHash() => r'49d6dbedfe397857ba85deb8c43d47c5d83047a1';

/// 平台管理员认证控制器。
///
/// 状态机：[AuthLoading] → [AuthAuthenticated] | [AuthError] | [AuthUnauthenticated]。
/// - [signin] 调用 SDK 登录，持久化 JWT，拉取 `/administrators/me`。
/// - [signout] 调用 SDK 登出，清除持久化 token。
/// - [restoreSession] 启动时从 SharedPreferences 恢复会话（若 token 仍有效）。
///
/// Copied from [AuthController].
@ProviderFor(AuthController)
final authControllerProvider =
    AutoDisposeNotifierProvider<AuthController, AuthState>.internal(
      AuthController.new,
      name: r'authControllerProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$authControllerHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

typedef _$AuthController = AutoDisposeNotifier<AuthState>;
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
