import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../pages/dashboard/dashboard_page.dart';
import '../pages/tenants/tenants_page.dart';
import '../pages/secrets/secrets_page.dart';
import '../pages/administrators/administrators_page.dart';
import '../pages/administrators/administrator_me_page.dart';
import '../pages/audits/audits_page.dart';
import '../pages/settings_page.dart';

/// 管理后台外壳：顶栏 + NavigationRail + 内容区。
///
/// nav 项按当前管理员权限集动态显示/隐藏（TODO: 接入 /administrators/me）。
class AdminShell extends StatefulWidget {
  const AdminShell({super.key});

  @override
  State<AdminShell> createState() => _AdminShellState();
}

class _AdminShellState extends State<AdminShell> {
  int _selectedIndex = 0;

  static const _destinations = [
    NavigationRailDestination(
      icon: Icon(FluentIcons.board_24_regular),
      selectedIcon: Icon(FluentIcons.board_24_filled),
      label: Text('Dashboard'),
    ),
    NavigationRailDestination(
      icon: Icon(FluentIcons.organization_24_regular),
      selectedIcon: Icon(FluentIcons.organization_24_filled),
      label: Text('Tenants'),
    ),
    NavigationRailDestination(
      icon: Icon(FluentIcons.key_24_regular),
      selectedIcon: Icon(FluentIcons.key_24_filled),
      label: Text('Secrets'),
    ),
    NavigationRailDestination(
      icon: Icon(FluentIcons.person_accounts_24_regular),
      selectedIcon: Icon(FluentIcons.person_accounts_24_filled),
      label: Text('Admins'),
    ),
    NavigationRailDestination(
      icon: Icon(FluentIcons.history_24_regular),
      selectedIcon: Icon(FluentIcons.history_24_filled),
      label: Text('Audits'),
    ),
    NavigationRailDestination(
      icon: Icon(FluentIcons.person_24_regular),
      selectedIcon: Icon(FluentIcons.person_24_filled),
      label: Text('Profile'),
    ),
    NavigationRailDestination(
      icon: Icon(FluentIcons.settings_24_regular),
      selectedIcon: Icon(FluentIcons.settings_24_filled),
      label: Text('Settings'),
    ),
  ];

  Widget _pageFor(int index) {
    switch (index) {
      case 0:
        return const DashboardPage();
      case 1:
        return const TenantsPage();
      case 2:
        return const SecretsPage();
      case 3:
        return const AdministratorsPage();
      case 4:
        return const AuditsPage();
      case 5:
        return const AdministratorMePage();
      case 6:
        return const SettingsPage();
      default:
        return const SizedBox.shrink();
    }
  }

  @override
  Widget build(BuildContext context) {
    final isWide = MediaQuery.of(context).size.width >= 900;
    return Scaffold(
      appBar: AppBar(
        title: const Text('OceanIAM Admin', key: Key('appbar-title')),
      ),
      body: Row(
        children: [
          NavigationRail(
            extended: isWide,
            minExtendedWidth: 200,
            selectedIndex: _selectedIndex,
            onDestinationSelected: (i) => setState(() => _selectedIndex = i),
            labelType: isWide
                ? NavigationRailLabelType.none
                : NavigationRailLabelType.all,
            destinations: _destinations,
          ),
          const VerticalDivider(thickness: 1, width: 1),
          Expanded(child: _pageFor(_selectedIndex)),
        ],
      ),
    );
  }
}
