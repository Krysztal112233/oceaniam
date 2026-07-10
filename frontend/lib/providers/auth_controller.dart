import 'package:flutter/foundation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:oceaniam_sdk/oceaniam_sdk.dart';
import 'oceaniam_client_provider.dart';
import 'shared_preferences_provider.dart';

part 'auth_controller.g.dart';

/// 认证状态。
@immutable
sealed class AuthState {
  const AuthState();
}

/// 未登录。
class AuthUnauthenticated extends AuthState {
  const AuthUnauthenticated();
}

/// 登录中（请求 /auth/tokens 或恢复会话）。
class AuthLoading extends AuthState {
  const AuthLoading();
}

/// 已登录，携带管理员档案。
class AuthAuthenticated extends AuthState {
  final AdministratorProfile profile;
  const AuthAuthenticated(this.profile);
}

/// 登录失败（凭证错误等），携带错误信息。
class AuthError extends AuthState {
  final String message;
  const AuthError(this.message);
}

/// 平台管理员认证控制器。
///
/// 状态机：[AuthLoading] → [AuthAuthenticated] | [AuthError] | [AuthUnauthenticated]。
/// - [signin] 调用 SDK 登录，持久化 JWT，拉取 `/administrators/me`。
/// - [signout] 调用 SDK 登出，清除持久化 token。
/// - [restoreSession] 启动时从 SharedPreferences 恢复会话（若 token 仍有效）。
@riverpod
class AuthController extends _$AuthController {
  SharedPreferences get _prefs => ref.read(sharedPreferencesProvider);
  OceanIAMClient get _client => ref.read(oceanIAMClientProvider);

  @override
  AuthState build() {
    return const AuthUnauthenticated();
  }

  /// 使用 name + password 登录。
  ///
  /// 系统管理员登录路径不会返回 MFA challenge（TOTP 仅在应用用户层）。
  Future<void> signin({required String name, required String password}) async {
    state = const AuthLoading();
    try {
      final response = await _client.signin(name, password);
      await _prefs.setString(kAuthTokenKey, response.jwt);
      final profile = await _client.getMyProfile();
      state = AuthAuthenticated(profile);
    } on OceanIAMError catch (e) {
      _client.setToken(null);
      await _prefs.remove(kAuthTokenKey);
      state = AuthError(_humanizeError(e));
    } catch (e) {
      _client.setToken(null);
      await _prefs.remove(kAuthTokenKey);
      state = AuthError('Sign in failed: $e');
    }
  }

  /// 登出并清除持久化 token。
  Future<void> signout() async {
    state = const AuthLoading();
    try {
      await _client.signout();
    } catch (_) {
      // 即便后端登出失败也清除本地状态
    } finally {
      _client.setToken(null);
      await _prefs.remove(kAuthTokenKey);
      state = const AuthUnauthenticated();
    }
  }

  /// 启动时尝试恢复会话。
  ///
  /// 仅在 SharedPreferences 中存在 token 时调用：用 `/administrators/me`
  /// 验证 token 仍有效；失败则清除 token 并回到未登录态。
  Future<void> restoreSession() async {
    final stored = _prefs.getString(kAuthTokenKey);
    if (stored == null || stored.isEmpty) {
      state = const AuthUnauthenticated();
      return;
    }
    _client.setToken(stored);
    state = const AuthLoading();
    try {
      final profile = await _client.getMyProfile();
      state = AuthAuthenticated(profile);
    } catch (e) {
      _client.setToken(null);
      await _prefs.remove(kAuthTokenKey);
      state = const AuthUnauthenticated();
    }
  }

  String _humanizeError(OceanIAMError e) {
    switch (e.statusCode) {
      case 401:
        return 'Invalid administrator name or password.';
      case 403:
        return 'Access forbidden.';
      case 400:
        return 'Malformed request: ${e.message}';
      default:
        return 'Sign in failed (${e.statusCode}): ${e.message}';
    }
  }
}
