import 'package:flutter/material.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../providers/shared_preferences_provider.dart';

part 'theme_controller.g.dart';

/// 持久化的主题模式控制器。
///
/// 值变更时同步写入 `SharedPreferences`，启动时读取已存储的偏好。
/// 默认 `ThemeMode.system`。
@riverpod
class ThemeController extends _$ThemeController {
  static const _key = 'theme_mode';

  @override
  ThemeMode build() {
    final prefs = ref.watch(sharedPreferencesProvider);
    final index = prefs.getInt(_key);
    return ThemeMode.values.firstWhere(
      (m) => m.index == index,
      orElse: () => ThemeMode.system,
    );
  }

  void setMode(ThemeMode mode) {
    if (state == mode) return;
    state = mode;
    ref.read(sharedPreferencesProvider).setInt(_key, mode.index);
  }

  static ThemeData light() {
    return ThemeData(
      useMaterial3: true,
      colorScheme: ColorScheme.fromSeed(
        seedColor: Colors.indigo,
        brightness: Brightness.light,
      ),
    );
  }

  static ThemeData dark() {
    return ThemeData(
      useMaterial3: true,
      colorScheme: ColorScheme.fromSeed(
        seedColor: Colors.indigo,
        brightness: Brightness.dark,
      ),
    );
  }
}
