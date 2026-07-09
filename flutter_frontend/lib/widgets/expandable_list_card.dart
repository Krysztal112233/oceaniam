import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

/// Card shell: ListTile header + chevron + AnimatedCrossFade body.
class ExpandableListCard extends StatelessWidget {
  final Widget leading;
  final Widget title;
  final Widget? subtitle;
  final bool isExpanded;
  final VoidCallback? onExpand;
  final Widget expandedChild;
  final EdgeInsetsGeometry margin;

  const ExpandableListCard({
    super.key,
    required this.leading,
    required this.title,
    this.subtitle,
    required this.isExpanded,
    this.onExpand,
    required this.expandedChild,
    this.margin = const EdgeInsets.only(bottom: 16),
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final expandEnabled = onExpand != null;

    return Card(
      margin: margin,
      clipBehavior: Clip.antiAlias,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          ListTile(
            leading: leading,
            title: title,
            subtitle: subtitle,
            trailing: expandEnabled
                ? Icon(
                    isExpanded
                        ? FluentIcons.chevron_up_24_regular
                        : FluentIcons.chevron_down_24_regular,
                    color: theme.colorScheme.onSurfaceVariant,
                  )
                : null,
            onTap: onExpand,
          ),
          AnimatedCrossFade(
            firstChild: const SizedBox.shrink(),
            secondChild: expandedChild,
            crossFadeState: isExpanded
                ? CrossFadeState.showSecond
                : CrossFadeState.showFirst,
            duration: const Duration(milliseconds: 220),
            firstCurve: Curves.easeInOut,
            secondCurve: Curves.easeInOut,
            sizeCurve: Curves.easeInOut,
            alignment: Alignment.topCenter,
          ),
        ],
      ),
    );
  }
}
