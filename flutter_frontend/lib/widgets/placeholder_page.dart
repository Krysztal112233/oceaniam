import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import 'admin_page_scaffold.dart';

/// 占位页骨架。页面骨架先行，实现后续填充。
class PlaceholderPage extends StatelessWidget {
  final String title;
  final String? description;
  final List<Widget>? actions;

  const PlaceholderPage({
    super.key,
    required this.title,
    this.description,
    this.actions,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return AdminPageScaffold(
      title: title,
      actions: actions,
      child: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                FluentIcons.box_toolbox_24_regular,
                size: 48,
                color: theme.colorScheme.outline,
              ),
              const SizedBox(height: 16),
              Text(title, style: theme.textTheme.headlineSmall),
              if (description != null) ...[
                const SizedBox(height: 8),
                Text(
                  description!,
                  style: theme.textTheme.bodyMedium,
                  textAlign: TextAlign.center,
                ),
              ],
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: 12,
                  vertical: 6,
                ),
                decoration: BoxDecoration(
                  color: theme.colorScheme.surfaceContainerHighest,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(
                  'TODO',
                  style: theme.textTheme.labelSmall?.copyWith(
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
