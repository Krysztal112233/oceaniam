import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../providers/application_providers.dart';
import '../../widgets/admin_page_scaffold.dart';
import 'application_card.dart';
import 'application_detail_page.dart';
import 'create_application_dialog.dart';

class ApplicationsPage extends ConsumerStatefulWidget {
  final String tenantId;

  const ApplicationsPage({super.key, required this.tenantId});

  @override
  ConsumerState<ApplicationsPage> createState() => _ApplicationsPageState();
}

class _ApplicationsPageState extends ConsumerState<ApplicationsPage> {
  String? _expandedApplicationId;

  @override
  Widget build(BuildContext context) {
    final isWide = MediaQuery.of(context).size.width >= 900;

    return _ApplicationListView(
      tenantId: widget.tenantId,
      expandedApplicationId: isWide ? _expandedApplicationId : null,
      onExpand: isWide
          ? (id) => setState(() {
              _expandedApplicationId = _expandedApplicationId == id ? null : id;
            })
          : null,
      onUsers: isWide ? null : _openUsersTab,
    );
  }

  void _openUsersTab(String applicationId) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => ApplicationDetailPage(
          tenantId: widget.tenantId,
          applicationId: applicationId,
          initialTabIndex: 1,
        ),
      ),
    );
  }
}

class _ApplicationListView extends ConsumerWidget {
  final String tenantId;
  final String? expandedApplicationId;
  final ValueChanged<String>? onExpand;
  final ValueChanged<String>? onUsers;

  const _ApplicationListView({
    required this.tenantId,
    this.expandedApplicationId,
    this.onExpand,
    this.onUsers,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appsAsync = ref.watch(applicationListProvider);

    return AdminPageScaffold(
      title: 'Applications',
      actions: [
        FilledButton.icon(
          key: const Key('create-application'),
          onPressed: () => _showCreateDialog(context, ref),
          icon: const Icon(FluentIcons.add_24_regular),
          label: const Text('New application'),
        ),
      ],
      child: appsAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('Failed to load applications: $e')),
        data: (apps) {
          if (apps.isEmpty) {
            return Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    FluentIcons.app_folder_24_regular,
                    size: 48,
                    color: Theme.of(context).colorScheme.outline,
                  ),
                  const SizedBox(height: 16),
                  Text(
                    'No applications yet',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 8),
                  FilledButton.tonalIcon(
                    onPressed: () => _showCreateDialog(context, ref),
                    icon: const Icon(FluentIcons.add_24_regular),
                    label: const Text('Create your first application'),
                  ),
                ],
              ),
            );
          }
          return ListView.builder(
            padding: const EdgeInsets.all(16),
            itemCount: apps.length,
            itemBuilder: (context, i) => ApplicationCard(
              application: apps[i],
              tenantId: tenantId,
              isExpanded: expandedApplicationId == apps[i].id,
              onExpand: onExpand != null ? () => onExpand!(apps[i].id) : null,
              onUsers: onUsers != null ? () => onUsers!(apps[i].id) : null,
            ),
          );
        },
      ),
    );
  }

  void _showCreateDialog(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (_) => CreateApplicationDialog(tenantId: tenantId),
    );
  }
}
