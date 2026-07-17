import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import 'admin_page_scaffold.dart';
import 'empty_state_illustration.dart';

/// 占位页骨架。页面骨架先行，实现后续填充。
class PlaceholderPage extends StatelessWidget {
  final String title;
  final String? description;
  final List<Widget>? leading;
  final List<Widget>? actions;

  const PlaceholderPage({
    super.key,
    required this.title,
    this.description,
    this.leading,
    this.actions,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return AdminPageScaffold(
      title: title,
      leading: leading,
      actions: actions,
      child: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              EmptyStateIllustration(
                icon: FluentIcons.box_toolbox_24_regular,
                title: title,
                message: description,
              ),
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
