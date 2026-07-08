import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../pages/dashboard/dashboard_page.dart';
import '../pages/tenants/tenants_page.dart';
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
    label: 'Tenants',
    icon: FluentIcons.organization_24_regular,
    selectedIcon: FluentIcons.organization_24_filled,
    page: const TenantsPage(),
    keySuffix: 'tenants',
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
            extended: isWide,
            selectedIndex: _selectedIndex,
            onDestinationSelected: (i) => setState(() => _selectedIndex = i),
            labelType: isWide
                ? NavigationRailLabelType.none
                : NavigationRailLabelType.selected,
            destinations: _navItems.map((n) => n.destination).toList(),
          ),
          const VerticalDivider(thickness: 1, width: 1),
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
