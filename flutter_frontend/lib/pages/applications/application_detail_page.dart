import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../widgets/admin_page_scaffold.dart';
import '../../widgets/placeholder_page.dart';

/// 应用详情页（占位实现）。
///
/// 后端：
/// - GET /tenants/{tenant_id}/applications/{application_id}（ApplicationRead）
/// - PATCH /tenants/{tenant_id}/applications/{application_id}（ApplicationPatch）
/// - DELETE /tenants/{tenant_id}/applications/{application_id}（ApplicationDelete）
/// - GET /tenants/{tenant_id}/applications/{application_id}/users（ApplicationUser list）
/// - POST /tenants/{tenant_id}/applications/{application_id}/users（ApplicationUserCreate）
class ApplicationDetailPage extends StatefulWidget {
  final String tenantId;
  final String applicationId;
  final int initialTabIndex;

  const ApplicationDetailPage({
    super.key,
    required this.tenantId,
    required this.applicationId,
    this.initialTabIndex = 0,
  });

  @override
  State<ApplicationDetailPage> createState() => _ApplicationDetailPageState();
}

class _ApplicationDetailPageState extends State<ApplicationDetailPage>
    with TickerProviderStateMixin {
  late TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(
      length: 3,
      vsync: this,
      initialIndex: widget.initialTabIndex,
    );
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AdminPageScaffold(
      title: 'Application ${widget.applicationId}',
      leading: [
        IconButton(
          icon: const Icon(FluentIcons.arrow_left_24_regular),
          onPressed: () => Navigator.of(context).maybePop(),
          tooltip: 'Back',
        ),
      ],
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          TabBar(
            controller: _tabController,
            isScrollable: true,
            tabs: const [
              Tab(text: 'Overview', icon: Icon(FluentIcons.info_24_regular)),
              Tab(text: 'Users', icon: Icon(FluentIcons.people_24_regular)),
              Tab(text: 'Secrets', icon: Icon(FluentIcons.key_24_regular)),
            ],
          ),
          Expanded(
            child: TabBarView(
              controller: _tabController,
              children: [
                PlaceholderPage(
                  title: 'Overview',
                  description:
                      'Application metadata and settings for ${widget.applicationId}.',
                ),
                PlaceholderPage(
                  title: 'Users',
                  description:
                      'Manage application users for ${widget.applicationId}.',
                ),
                PlaceholderPage(
                  title: 'Secrets',
                  description:
                      'Manage secret bindings for ${widget.applicationId}.',
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
