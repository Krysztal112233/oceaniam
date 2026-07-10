import 'package:flutter/material.dart';

/// One segment in a [SegmentedExpandPanel].
class ExpandPanelTab {
  final IconData icon;
  final String label;
  final WidgetBuilder builder;

  const ExpandPanelTab({
    required this.icon,
    required this.label,
    required this.builder,
  });
}

/// Divider + SegmentedButton + tab content for expandable list cards.
class SegmentedExpandPanel extends StatefulWidget {
  final List<ExpandPanelTab> tabs;
  final int initialIndex;

  const SegmentedExpandPanel({
    super.key,
    required this.tabs,
    this.initialIndex = 0,
  }) : assert(tabs.length > 0);

  @override
  State<SegmentedExpandPanel> createState() => _SegmentedExpandPanelState();
}

class _SegmentedExpandPanelState extends State<SegmentedExpandPanel> {
  late int _selectedTab;

  @override
  void initState() {
    super.initState();
    _selectedTab = widget.initialIndex.clamp(0, widget.tabs.length - 1);
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final content = widget.tabs[_selectedTab].builder(context);

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Divider(height: 1, color: theme.colorScheme.outlineVariant),
        Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              SegmentedButton<int>(
                showSelectedIcon: false,
                segments: [
                  for (var i = 0; i < widget.tabs.length; i++)
                    ButtonSegment<int>(
                      value: i,
                      icon: Icon(widget.tabs[i].icon, size: 18),
                      label: Text(widget.tabs[i].label),
                    ),
                ],
                selected: {_selectedTab},
                onSelectionChanged: (v) =>
                    setState(() => _selectedTab = v.first),
                emptySelectionAllowed: false,
              ),
              const SizedBox(height: 16),
              content,
            ],
          ),
        ),
      ],
    );
  }
}
