import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:dropdown_button2/dropdown_button2.dart';

import '../pages/dashboard/dashboard_page.dart';
import '../pages/secrets/secrets_page.dart';
import '../pages/administrators/administrators_page.dart';
import '../pages/administrators/administrator_me_page.dart';
import '../pages/audits/audits_page.dart';
import '../pages/settings_page.dart';
import '../providers/tenant_providers.dart';
import '../providers/oceaniam_client_provider.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

class _NavItem {
  final String label;
  final IconData icon;
  final IconData selectedIcon;
  final Widget page;

  const _NavItem({
    required this.label,
    required this.icon,
    required this.selectedIcon,
    required this.page,
  });

  NavigationRailDestination get destination => NavigationRailDestination(
    icon: Icon(icon),
    selectedIcon: Icon(selectedIcon),
    label: Text(label),
  );
}

final _navItems = <_NavItem>[
  _NavItem(
    label: 'Dashboard',
    icon: FluentIcons.board_24_regular,
    selectedIcon: FluentIcons.board_24_filled,
    page: const DashboardPage(),
  ),
  _NavItem(
    label: 'Secrets',
    icon: FluentIcons.key_24_regular,
    selectedIcon: FluentIcons.key_24_filled,
    page: const SecretsPage(),
  ),
  _NavItem(
    label: 'Admins',
    icon: FluentIcons.person_accounts_24_regular,
    selectedIcon: FluentIcons.person_accounts_24_filled,
    page: const AdministratorsPage(),
  ),
  _NavItem(
    label: 'Audits',
    icon: FluentIcons.history_24_regular,
    selectedIcon: FluentIcons.history_24_filled,
    page: const AuditsPage(),
  ),
  _NavItem(
    label: 'Profile',
    icon: FluentIcons.person_24_regular,
    selectedIcon: FluentIcons.person_24_filled,
    page: const AdministratorMePage(),
  ),
  _NavItem(
    label: 'Settings',
    icon: FluentIcons.settings_24_regular,
    selectedIcon: FluentIcons.settings_24_filled,
    page: const SettingsPage(),
  ),
];

class AdminShell extends ConsumerStatefulWidget {
  const AdminShell({super.key});

  @override
  ConsumerState<AdminShell> createState() => _AdminShellState();
}

class _AdminShellState extends ConsumerState<AdminShell> {
  int _selectedIndex = 0;

  @override
  Widget build(BuildContext context) {
    final isWide = MediaQuery.of(context).size.width >= 900;

    return Scaffold(
      body: Row(
        children: [
          NavigationRail(
            elevation: 4,
            extended: isWide,
            selectedIndex: _selectedIndex,
            onDestinationSelected: (i) => setState(() => _selectedIndex = i),
            labelType: isWide
                ? NavigationRailLabelType.none
                : NavigationRailLabelType.selected,
            leading: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const SizedBox(height: 8),
                _TenantSwitcher(extended: isWide),
                Container(height: 1, color: Theme.of(context).dividerColor),
              ],
            ),
            destinations: _navItems.map((n) => n.destination).toList(),
          ),
          Expanded(
            child: AnimatedSwitcher(
              duration: const Duration(milliseconds: 200),
              child: _selectedIndex < _navItems.length
                  ? _navItems[_selectedIndex].page
                  : const SizedBox.shrink(),
            ),
          ),
        ],
      ),
    );
  }
}

class _TenantSwitcher extends ConsumerStatefulWidget {
  final bool extended;

  const _TenantSwitcher({required this.extended});

  @override
  ConsumerState<_TenantSwitcher> createState() => _TenantSwitcherState();
}

class _TenantSwitcherState extends ConsumerState<_TenantSwitcher> {
  late final ValueNotifier<String?> _valueNotifier;

  @override
  void initState() {
    super.initState();
    _valueNotifier = ValueNotifier(null);
  }

  @override
  void dispose() {
    _valueNotifier.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final currentId = ref.watch(currentTenantIdProvider);
    _valueNotifier.value = currentId;

    final theme = Theme.of(context);
    final tenant = ref.watch(currentTenantProvider);
    final tenantName = tenant?.id ?? 'No tenant';
    final tenantsAsync = ref.watch(tenantListProvider);

    final items = <DropdownItem<String>>[];
    if (tenantsAsync.valueOrNull != null) {
      for (final t in tenantsAsync.valueOrNull!) {
        items.add(_buildTenantItem(t, currentId));
      }
    }
    items.add(_buildCreateTenantItem(theme));

    return Padding(
      padding: widget.extended
          ? const EdgeInsets.fromLTRB(12, 0, 12, 8)
          : const EdgeInsets.only(bottom: 8),
      child: DropdownButton2<String>(
        underline: const SizedBox.shrink(),
        customButton: _buildButton(theme, tenantName),
        valueListenable: _valueNotifier,
        dropdownStyleData: DropdownStyleData(
          direction: DropdownDirection.right,
          offset: const Offset(4, 0),
          maxHeight: 300,
          width: 280,
          elevation: 8,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(12),
            color: theme.colorScheme.surfaceContainerHigh,
          ),
          padding: const EdgeInsets.symmetric(vertical: 4),
        ),
        items: items,
        onChanged: (id) {
          if (id == '__create__') {
            Future.microtask(_showCreateTenantDialog);
            return;
          }
          if (id != null) {
            ref.read(currentTenantIdProvider.notifier).select(id);
          }
        },
      ),
    );
  }

  DropdownItem<String> _buildTenantItem(Tenant t, String? currentId) {
    final theme = Theme.of(context);
    return DropdownItem<String>(
      value: t.id,
      child: Row(
        children: [
          _OrgIcon(size: 28, iconSize: 16),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(t.id, overflow: TextOverflow.ellipsis),
                if (t.comment != null && t.comment!.isNotEmpty)
                  Text(
                    t.comment!,
                    style: theme.textTheme.labelSmall?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                    overflow: TextOverflow.ellipsis,
                    maxLines: 1,
                  ),
              ],
            ),
          ),
          if (t.id == currentId)
            Icon(
              FluentIcons.checkmark_24_filled,
              size: 18,
              color: theme.colorScheme.primary,
            ),
        ],
      ),
    );
  }

  DropdownItem<String> _buildCreateTenantItem(ThemeData theme) {
    return DropdownItem<String>(
      value: '__create__',
      child: Row(
        children: [
          Icon(
            FluentIcons.add_24_regular,
            size: 18,
            color: theme.colorScheme.primary,
          ),
          const SizedBox(width: 10),
          Text(
            'Create Tenant',
            style: TextStyle(color: theme.colorScheme.primary),
          ),
        ],
      ),
    );
  }

  Widget _buildButton(ThemeData theme, String tenantName) {
    return widget.extended
        ? _buildExtendedButton(theme, tenantName)
        : _buildCollapsedButton(theme);
  }

  Widget _buildExtendedButton(ThemeData theme, String tenantName) {
    return SizedBox(
      width: 220,
      child: Card(
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: theme.colorScheme.outlineVariant),
        ),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          child: Row(
            children: [
              _OrgIcon(size: 32, iconSize: 18),
              const SizedBox(width: 10),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text('Current Tenant'),
                    Text(tenantName, overflow: TextOverflow.ellipsis),
                  ],
                ),
              ),
              const SizedBox(width: 4),
              Icon(
                FluentIcons.chevron_down_24_regular,
                size: 16,
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildCollapsedButton(ThemeData theme) {
    return Container(
      width: 48,
      height: 48,
      decoration: BoxDecoration(
        color: theme.colorScheme.primaryContainer,
        borderRadius: BorderRadius.circular(8),
        boxShadow: [
          BoxShadow(
            color: theme.colorScheme.shadow.withValues(alpha: 0.3),
            blurRadius: 2,
            offset: Offset(0, 2),
          ),
        ],
      ),
      child: const _OrgIcon(size: 20, iconSize: 20),
    );
  }

  Future<void> _showCreateTenantDialog() async {
    final tenant = await showDialog<Tenant>(
      context: context,
      builder: (_) => const _CreateTenantDialog(),
    );
    if (tenant != null) {
      ref.read(currentTenantIdProvider.notifier).select(tenant.id);
      ref.invalidate(tenantListProvider);
    }
  }
}

class _CreateTenantDialog extends ConsumerStatefulWidget {
  const _CreateTenantDialog();

  @override
  ConsumerState<_CreateTenantDialog> createState() =>
      _CreateTenantDialogState();
}

class _CreateTenantDialogState extends ConsumerState<_CreateTenantDialog> {
  final _controller = TextEditingController();
  bool _loading = false;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return AlertDialog(
      backgroundColor: theme.colorScheme.surfaceContainerHigh,
      surfaceTintColor: theme.colorScheme.surfaceTint,
      elevation: 3,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(28)),
      titlePadding: const EdgeInsets.fromLTRB(24, 24, 24, 0),
      contentPadding: const EdgeInsets.fromLTRB(24, 20, 24, 0),
      actionsPadding: const EdgeInsets.fromLTRB(24, 8, 24, 16),
      title: const Text('Create Tenant'),
      content: TextField(
        controller: _controller,
        autofocus: true,
        enabled: !_loading,
        decoration: const InputDecoration(
          labelText: 'Comment',
          hintText: 'Optional description for this tenant',
        ),
        onSubmitted: _loading ? null : (_) => _create(),
      ),
      actions: [
        TextButton(
          onPressed: _loading ? null : () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: _loading ? null : _create,
          child: _loading
              ? const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Text('Create'),
        ),
      ],
    );
  }

  Future<void> _create() async {
    setState(() => _loading = true);
    try {
      final client = ref.read(oceanIAMClientProvider);
      final tenant = await client.createTenant(
        comment: _controller.text.isNotEmpty ? _controller.text : null,
      );
      if (mounted) Navigator.of(context).pop(tenant);
    } catch (e) {
      if (mounted) {
        setState(() => _loading = false);
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Failed to create tenant: $e')));
      }
    }
  }
}

class _OrgIcon extends StatelessWidget {
  final double size;
  final double iconSize;

  const _OrgIcon({required this.size, required this.iconSize});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        color: theme.colorScheme.primaryContainer,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Icon(
        FluentIcons.organization_24_regular,
        size: iconSize,
        color: theme.colorScheme.onPrimaryContainer,
      ),
    );
  }
}
