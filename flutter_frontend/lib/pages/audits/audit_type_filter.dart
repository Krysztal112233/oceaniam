import 'package:dropdown_button2/dropdown_button2.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/material.dart';

import '../../providers/audit_providers.dart';

/// Sentinel for "All types" — DropdownButton2 items need non-null values.
const kAllAuditTypesValue = '__all__';

/// Searchable audit-type filter chip (DropdownButton2).
class AuditTypeFilter extends StatefulWidget {
  final String? auditType;
  final ValueChanged<String?> onChanged;

  const AuditTypeFilter({
    super.key,
    required this.auditType,
    required this.onChanged,
  });

  @override
  State<AuditTypeFilter> createState() => _AuditTypeFilterState();
}

class _AuditTypeFilterState extends State<AuditTypeFilter> {
  static const _searchBarHeight = 48.0;

  late final ValueNotifier<String> _valueNotifier;
  late final TextEditingController _searchController;

  @override
  void initState() {
    super.initState();
    _valueNotifier = ValueNotifier(widget.auditType ?? kAllAuditTypesValue);
    _searchController = TextEditingController();
  }

  @override
  void didUpdateWidget(covariant AuditTypeFilter oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.auditType != widget.auditType) {
      _valueNotifier.value = widget.auditType ?? kAllAuditTypesValue;
    }
  }

  @override
  void dispose() {
    _valueNotifier.dispose();
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final selected = widget.auditType;

    final items = <DropdownItem<String>>[
      DropdownItem<String>(
        value: kAllAuditTypesValue,
        child: Row(
          children: [
            const Expanded(child: Text('All types')),
            if (selected == null)
              Icon(
                FluentIcons.checkmark_24_filled,
                size: 18,
                color: theme.colorScheme.primary,
              ),
          ],
        ),
      ),
      for (final type in kAuditTypes)
        DropdownItem<String>(
          value: type,
          child: Row(
            children: [
              Expanded(child: Text(type)),
              if (selected == type)
                Icon(
                  FluentIcons.checkmark_24_filled,
                  size: 18,
                  color: theme.colorScheme.primary,
                ),
            ],
          ),
        ),
    ];

    return DropdownButton2<String>(
      underline: const SizedBox.shrink(),
      customButton: InputChip(
        avatar: Icon(
          FluentIcons.filter_24_regular,
          size: 18,
          color: theme.colorScheme.onSurfaceVariant,
        ),
        label: Text(selected ?? 'All types'),
        onDeleted: selected == null ? null : () => widget.onChanged(null),
        deleteIcon: const Icon(FluentIcons.dismiss_12_regular, size: 14),
      ),
      buttonStyleData: ButtonStyleData(
        decoration: BoxDecoration(borderRadius: BorderRadius.circular(8)),
      ),
      valueListenable: _valueNotifier,
      items: items,
      onChanged: (value) {
        if (value == null) return;
        _searchController.clear();
        widget.onChanged(value == kAllAuditTypesValue ? null : value);
      },
      onMenuStateChange: (isOpen) {
        if (!isOpen) _searchController.clear();
      },
      dropdownStyleData: DropdownStyleData(
        maxHeight: 360,
        width: 280,
        elevation: 8,
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(12),
          color: theme.colorScheme.surfaceContainerHigh,
        ),
        padding: const EdgeInsets.only(bottom: 4),
      ),
      dropdownSearchData: DropdownSearchData(
        searchController: _searchController,
        searchBarWidgetHeight: _searchBarHeight,
        searchBarWidget: Container(
          height: _searchBarHeight,
          padding: const EdgeInsets.fromLTRB(12, 8, 12, 4),
          child: TextField(
            controller: _searchController,
            autofocus: true,
            decoration: InputDecoration(
              isDense: true,
              hintText: 'Search types…',
              prefixIcon: const Icon(FluentIcons.search_24_regular, size: 18),
              prefixIconConstraints: const BoxConstraints(
                minWidth: 36,
                minHeight: 36,
              ),
              contentPadding: const EdgeInsets.symmetric(
                horizontal: 8,
                vertical: 8,
              ),
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
              ),
            ),
          ),
        ),
        noResultsWidget: Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            'No matching types',
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
        ),
        searchMatchFn: (item, search) {
          final q = search.trim().toLowerCase();
          if (q.isEmpty) return true;
          final value = item.value;
          if (value == null) return false;
          if (value == kAllAuditTypesValue) {
            return 'all types'.contains(q) || 'all'.contains(q);
          }
          return value.toLowerCase().contains(q);
        },
      ),
    );
  }
}
