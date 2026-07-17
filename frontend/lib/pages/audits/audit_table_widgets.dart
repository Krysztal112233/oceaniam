import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/material.dart';

class AuditTypeBadge extends StatelessWidget {
  final String type;

  const AuditTypeBadge(this.type, {super.key});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Align(
        alignment: Alignment.centerLeft,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            color: theme.colorScheme.secondaryContainer,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Text(
            type,
            overflow: TextOverflow.ellipsis,
            style: theme.textTheme.labelMedium?.copyWith(
              color: theme.colorScheme.onSecondaryContainer,
            ),
          ),
        ),
      ),
    );
  }
}

class AuditHeaderCell extends StatelessWidget {
  final String label;
  final TextStyle? style;

  const AuditHeaderCell({super.key, required this.label, this.style});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Align(
        alignment: Alignment.centerLeft,
        child: Text(label, style: style, overflow: TextOverflow.ellipsis),
      ),
    );
  }
}

class AuditPaginationBar extends StatelessWidget {
  final int page;
  final int pageSize;
  final int total;
  final bool enabled;
  final VoidCallback? onPrevious;
  final VoidCallback? onNext;

  const AuditPaginationBar({
    super.key,
    required this.page,
    required this.pageSize,
    required this.total,
    this.enabled = true,
    this.onPrevious,
    this.onNext,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final start = total == 0 ? 0 : (page - 1) * pageSize + 1;
    final end = (page * pageSize).clamp(0, total);

    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          Text('$start–$end of $total', style: theme.textTheme.bodySmall),
          IconButton(
            icon: const Icon(FluentIcons.chevron_left_24_regular),
            tooltip: 'Previous page',
            onPressed: enabled ? onPrevious : null,
          ),
          IconButton(
            icon: const Icon(FluentIcons.chevron_right_24_regular),
            tooltip: 'Next page',
            onPressed: enabled ? onNext : null,
          ),
        ],
      ),
    );
  }
}

class AuditEmptyMessage extends StatelessWidget {
  final IconData icon;
  final String message;

  const AuditEmptyMessage({
    super.key,
    required this.icon,
    required this.message,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 40, color: theme.colorScheme.outline),
          const SizedBox(height: 12),
          Text(message, style: theme.textTheme.bodyMedium),
        ],
      ),
    );
  }
}
