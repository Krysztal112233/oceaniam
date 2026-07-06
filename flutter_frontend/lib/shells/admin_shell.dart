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
import '../providers/auth_controller.dart';
import '../theme/theme_controller.dart';
import '../widgets/theme_toggle.dart';

/// 管理后台外壳：顶栏 + NavigationRail + 内容区。
///
/// nav 项按当前管理员权限集动态显示/隐藏（TODO: 接入 /administrators/me）。
/// NavigationRail 底部（trailing）放置主题切换按钮，持久化到 SharedPreferences。
class AdminShell extends ConsumerStatefulWidget {
  const AdminShell({super.key});

  @override
  ConsumerState<AdminShell> createState() => _AdminShellState();
}

class _AdminShellState extends ConsumerState<AdminShell> {
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
        return const DashboardPage(key: ValueKey('page-dashboard'));
      case 1:
        return const TenantsPage(key: ValueKey('page-tenants'));
      case 2:
        return const SecretsPage(key: ValueKey('page-secrets'));
      case 3:
        return const AdministratorsPage(key: ValueKey('page-admins'));
      case 4:
        return const AuditsPage(key: ValueKey('page-audits'));
      case 5:
        return const AdministratorMePage(key: ValueKey('page-profile'));
      case 6:
        return const SettingsPage(key: ValueKey('page-settings'));
      default:
        return const SizedBox.shrink(key: ValueKey('page-empty'));
    }
  }

  @override
  Widget build(BuildContext context) {
    final isWide = MediaQuery.of(context).size.width >= 900;
    final themeMode = ref.watch(themeControllerProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('OceanIAM Admin', key: Key('appbar-title')),
        actions: [
          IconButton(
            key: const Key('signout-button'),
            tooltip: 'Sign out',
            icon: const Icon(FluentIcons.arrow_exit_20_regular),
            onPressed: () async {
              await ref.read(authControllerProvider.notifier).signout();
            },
          ),
        ],
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
            trailing: Expanded(
              child: Align(
                alignment: Alignment.bottomCenter,
                child: ThemeToggle(
                  mode: themeMode,
                  onChanged: (m) =>
                      ref.read(themeControllerProvider.notifier).setMode(m),
                ),
              ),
            ),
          ),
          const VerticalDivider(thickness: 1, width: 1),
          Expanded(
            child: AnimatedSwitcher(
              duration: const Duration(milliseconds: 200),
              switchInCurve: Curves.easeOut,
              switchOutCurve: Curves.easeIn,
              transitionBuilder: (child, animation) {
                return FadeTransition(opacity: animation, child: child);
              },
              child: _pageFor(_selectedIndex),
            ),
          ),
        ],
      ),
    );
  }
}
