import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'shared_preferences_provider.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

part 'oceaniam_client_provider.g.dart';

/// SharedPreferences 中持久化 JWT 的键名。
const kAuthTokenKey = 'auth_token';

/// 后端 base URL。
///
/// 开发环境默认指向本地 Axum 服务；生产环境应通过 --dart-define 或环境变量覆盖。
const kBackendBaseUrl = String.fromEnvironment(
  'OCEANIAM_BACKEND_URL',
  defaultValue: 'http://localhost:8000',
);

/// 单例 [OceanIAMClient] provider。
///
/// 启动时从 [SharedPreferences] 读取已保存的 JWT 并注入到 client，
/// 使应用刷新后仍处于已登录状态。token 的写入由 [AuthController] 负责。
@Riverpod(keepAlive: true)
OceanIAMClient oceanIAMClient(Ref ref) {
  final prefs = ref.watch(sharedPreferencesProvider);
  final client = OceanIAMClient(baseUrl: kBackendBaseUrl);
  final stored = prefs.getString(kAuthTokenKey);
  if (stored != null && stored.isNotEmpty) {
    client.setToken(stored);
  }
  ref.onDispose(client.dispose);
  return client;
}
