import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

part 'shared_preferences_provider.g.dart';

/// 单例 SharedPreferences 实例 provider。
///
/// 初始化是异步的，应用启动时在 `main` 中 prewarm 以避免首次读取时阻塞。
@Riverpod(keepAlive: true)
SharedPreferences sharedPreferences(Ref ref) {
  throw UnimplementedError('Must be overridden in ProviderScope');
}
