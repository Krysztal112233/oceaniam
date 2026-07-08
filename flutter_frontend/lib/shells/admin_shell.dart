import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../pages/dashboard/dashboard_page.dart';
import '../pages/secrets/secrets_page.dart';
import '../pages/administrators/administrators_page.dart';
import '../pages/administrators/administrator_me_page.dart';
import '../pages/audits/audits_page.dart';
import '../pages/settings_page.dart';

class _NavItem {
  final String label;
  final IconData icon;
  final IconData selectedIcon;
  final Widget page;
  final String keySuffix;

  const _NavItem({
    required this.label,
    required this.icon,
    required this.selectedIcon,
    required this.page,
    required this.keySuffix,
  });

  NavigationRailDestination get destination => NavigationRailDestination(
    icon: Icon(icon),
    selectedIcon: Icon(selectedIcon),
    label: Text(label),
  );

  ValueKey<String> get pageKey => ValueKey('page-$keySuffix');
}

final _navItems = <_NavItem>[
  _NavItem(
    label: 'Dashboard',
    icon: FluentIcons.board_24_regular,
    selectedIcon: FluentIcons.board_24_filled,
    page: const DashboardPage(),
    keySuffix: 'dashboard',
  ),
  _NavItem(
    label: 'Secrets',
    icon: FluentIcons.key_24_regular,
    selectedIcon: FluentIcons.key_24_filled,
    page: const SecretsPage(),
    keySuffix: 'secrets',
  ),
  _NavItem(
    label: 'Admins',
    icon: FluentIcons.person_accounts_24_regular,
    selectedIcon: FluentIcons.person_accounts_24_filled,
    page: const AdministratorsPage(),
    keySuffix: 'admins',
  ),
  _NavItem(
    label: 'Audits',
    icon: FluentIcons.history_24_regular,
    selectedIcon: FluentIcons.history_24_filled,
    page: const AuditsPage(),
    keySuffix: 'audits',
  ),
  _NavItem(
    label: 'Profile',
    icon: FluentIcons.person_24_regular,
    selectedIcon: FluentIcons.person_24_filled,
    page: const AdministratorMePage(),
    keySuffix: 'profile',
  ),
  _NavItem(
    label: 'Settings',
    icon: FluentIcons.settings_24_regular,
    selectedIcon: FluentIcons.settings_24_filled,
    page: const SettingsPage(),
    keySuffix: 'settings',
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

class _TenantSwitcher extends StatelessWidget {
  final bool extended;

  const _TenantSwitcher({required this.extended});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    if (extended) {
      return Padding(
        padding: const EdgeInsets.fromLTRB(12, 0, 12, 8),
        child: Card(
          elevation: 0,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
            side: BorderSide(color: theme.colorScheme.outlineVariant),
          ),
          child: InkWell(
            onTap: () {},
            borderRadius: BorderRadius.circular(12),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Container(
                    width: 32,
                    height: 32,
                    decoration: BoxDecoration(
                      color: theme.colorScheme.primaryContainer,
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Icon(
                      FluentIcons.organization_24_regular,
                      size: 18,
                      color: theme.colorScheme.onPrimaryContainer,
                    ),
                  ),
                  const SizedBox(width: 10),
                  Flexible(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text(
                          'Current Tenant',
                          style: theme.textTheme.labelSmall?.copyWith(
                            color: theme.colorScheme.onSurfaceVariant,
                          ),
                        ),
                        Text(
                          'Click to switch',
                          style: theme.textTheme.bodyMedium?.copyWith(
                            fontWeight: FontWeight.w500,
                          ),
                        ),
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
        ),
      );
    }

    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: InkWell(
        onTap: () {},
        borderRadius: BorderRadius.circular(8),
        child: Container(
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
          child: Icon(
            FluentIcons.organization_24_regular,
            size: 20,
            color: theme.colorScheme.onPrimaryContainer,
          ),
        ),
      ),
    );
  }
}
