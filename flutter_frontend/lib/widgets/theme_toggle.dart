import 'package:flutter/material.dart';

/// NavigationRail 底部的主题切换按钮（light / dark / system 三态循环）。
///
/// 放在 rail 的 trailing 位置，不占用 destinations 槽位。
class ThemeToggle extends StatelessWidget {
  final ThemeMode mode;
  final ValueChanged<ThemeMode> onChanged;

  const ThemeToggle({super.key, required this.mode, required this.onChanged});

  static const _cycle = [ThemeMode.system, ThemeMode.light, ThemeMode.dark];

  ThemeMode _next(ThemeMode current) {
    final i = _cycle.indexOf(current);
    return _cycle[(i + 1) % _cycle.length];
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final (label, icon) = switch (mode) {
      ThemeMode.system => (
        'System',
        theme.brightness == Brightness.dark
            ? Icons.brightness_3
            : Icons.brightness_5,
      ),
      ThemeMode.light => ('Light', Icons.brightness_7),
      ThemeMode.dark => ('Dark', Icons.brightness_2),
    };

    final isWide = MediaQuery.of(context).size.width >= 900;

    return isWide
        ? Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            child: Align(
              alignment: Alignment.centerLeft,
              child: TextButton.icon(
                key: const Key('theme-toggle'),
                onPressed: () => onChanged(_next(mode)),
                icon: Icon(icon),
                label: Text(label),
              ),
            ),
          )
        : IconButton(
            key: const Key('theme-toggle'),
            tooltip: label,
            onPressed: () => onChanged(_next(mode)),
            icon: Icon(icon),
          );
  }
}
