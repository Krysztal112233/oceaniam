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
              LayoutBuilder(
                builder: (context, constraints) {
                  // Roughly 140dp keeps an icon and a useful text label from
                  // being clipped. Compact panels switch to a picker instead
                  // of hiding later tabs in a horizontal scroller.
                  final usePicker =
                      constraints.maxWidth < widget.tabs.length * 140;
                  if (usePicker) {
                    return InputDecorator(
                      decoration: const InputDecoration(
                        labelText: 'Section',
                        border: OutlineInputBorder(),
                        contentPadding: EdgeInsets.symmetric(
                          horizontal: 12,
                          vertical: 4,
                        ),
                      ),
                      child: DropdownButtonHideUnderline(
                        child: DropdownButton<int>(
                          key: const Key('expand-panel-section-picker'),
                          value: _selectedTab,
                          isExpanded: true,
                          items: [
                            for (var i = 0; i < widget.tabs.length; i++)
                              DropdownMenuItem<int>(
                                value: i,
                                child: Row(
                                  children: [
                                    Icon(widget.tabs[i].icon, size: 18),
                                    const SizedBox(width: 8),
                                    Text(widget.tabs[i].label),
                                  ],
                                ),
                              ),
                          ],
                          onChanged: (value) {
                            if (value != null) {
                              setState(() => _selectedTab = value);
                            }
                          },
                        ),
                      ),
                    );
                  }

                  return SegmentedButton<int>(
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
                  );
                },
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
