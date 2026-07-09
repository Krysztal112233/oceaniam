import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/application_providers.dart';
import '../../providers/oceaniam_client_provider.dart';
import '../../widgets/admin_page_scaffold.dart';
import 'application_detail_page.dart';

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
              _expandedApplicationId =
                  _expandedApplicationId == id ? null : id;
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
            itemBuilder: (context, i) => _ApplicationCard(
              application: apps[i],
              isExpanded: expandedApplicationId == apps[i].id,
              onExpand: onExpand != null
                  ? () => onExpand!(apps[i].id)
                  : null,
              onDelete: () => _deleteApplication(context, ref, apps[i]),
              onUsers: onUsers != null
                  ? () => onUsers!(apps[i].id)
                  : null,
            ),
          );
        },
      ),
    );
  }

  void _showCreateDialog(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (_) => _CreateApplicationDialog(tenantId: tenantId),
    );
  }

  Future<void> _deleteApplication(
    BuildContext context,
    WidgetRef ref,
    Application app,
  ) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Delete application'),
        content: Text('Are you sure you want to delete "${app.id}"?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            style: FilledButton.styleFrom(
              backgroundColor: Theme.of(context).colorScheme.error,
            ),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    try {
      final client = ref.read(oceanIAMClientProvider);
      await client.deleteApplication(tenantId, app.id);
      ref.invalidate(applicationListProvider);
      if (context.mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Deleted "${app.id}"')));
      }
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Failed to delete: $e')));
      }
    }
  }
}

class _ApplicationCard extends StatelessWidget {
  final Application application;
  final bool isExpanded;
  final VoidCallback? onExpand;
  final VoidCallback onDelete;
  final VoidCallback? onUsers;

  const _ApplicationCard({
    required this.application,
    required this.isExpanded,
    this.onExpand,
    required this.onDelete,
    this.onUsers,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final expandEnabled = onExpand != null;

    Widget cardContent = Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        ListTile(
          leading: CircleAvatar(
            backgroundColor: theme.colorScheme.secondaryContainer,
            child: Icon(
              FluentIcons.app_folder_24_regular,
              color: theme.colorScheme.onSecondaryContainer,
            ),
          ),
          title: Text(application.id),
          subtitle: application.comment != null &&
                  application.comment!.isNotEmpty
              ? Text(application.comment!)
              : null,
          trailing: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (onUsers != null)
                IconButton(
                  icon: Icon(
                    FluentIcons.people_24_regular,
                    color: theme.colorScheme.primary,
                  ),
                  onPressed: onUsers,
                  tooltip: 'Manage users',
                ),
              IconButton(
                icon: Icon(
                  FluentIcons.delete_24_regular,
                  color: theme.colorScheme.error,
                ),
                onPressed: onDelete,
              ),
              if (expandEnabled)
                Padding(
                  padding: const EdgeInsets.only(left: 4),
                  child: Icon(
                    isExpanded
                        ? FluentIcons.chevron_up_24_regular
                        : FluentIcons.chevron_down_24_regular,
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                ),
            ],
          ),
          onTap: expandEnabled ? onExpand : null,
        ),
        AnimatedCrossFade(
          firstChild: const SizedBox.shrink(),
          secondChild: _ApplicationUsersPanel(applicationId: application.id),
          crossFadeState: isExpanded
              ? CrossFadeState.showSecond
              : CrossFadeState.showFirst,
          duration: const Duration(milliseconds: 220),
          firstCurve: Curves.easeInOut,
          secondCurve: Curves.easeInOut,
          sizeCurve: Curves.easeInOut,
          alignment: Alignment.topCenter,
        ),
      ],
    );

    return Card(
      margin: const EdgeInsets.only(bottom: 16),
      child: cardContent,
    );
  }
}

class _ApplicationUsersPanel extends StatelessWidget {
  final String applicationId;

  const _ApplicationUsersPanel({required this.applicationId});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Divider(height: 1, color: theme.colorScheme.outlineVariant),
        Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                children: [
                  Icon(
                    FluentIcons.people_24_regular,
                    size: 20,
                    color: theme.colorScheme.primary,
                  ),
                  const SizedBox(width: 8),
                  Text(
                    'Users',
                    style: theme.textTheme.titleMedium,
                  ),
                  const Spacer(),
                  FilledButton.tonalIcon(
                    onPressed: () {
                      // TODO: implement create user
                    },
                    icon: const Icon(FluentIcons.add_24_regular),
                    label: const Text('New user'),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 24,
                ),
                decoration: BoxDecoration(
                  color: theme.colorScheme.surfaceContainerHighest,
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Column(
                  children: [
                    Icon(
                      FluentIcons.people_24_regular,
                      size: 40,
                      color: theme.colorScheme.outline,
                    ),
                    const SizedBox(height: 12),
                    Text(
                      'User management for $applicationId',
                      style: theme.textTheme.bodyMedium,
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 8),
                    Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 12,
                        vertical: 4,
                      ),
                      decoration: BoxDecoration(
                        color: theme.colorScheme.surfaceContainer,
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Text(
                        'TODO',
                        style: theme.textTheme.labelSmall?.copyWith(
                          color: theme.colorScheme.onSurfaceVariant,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _CreateApplicationDialog extends ConsumerStatefulWidget {
  final String tenantId;

  const _CreateApplicationDialog({required this.tenantId});

  @override
  ConsumerState<_CreateApplicationDialog> createState() =>
      _CreateApplicationDialogState();
}

class _CreateApplicationDialogState
    extends ConsumerState<_CreateApplicationDialog> {
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
      title: const Text('Create Application'),
      content: TextField(
        controller: _controller,
        autofocus: true,
        enabled: !_loading,
        decoration: const InputDecoration(
          labelText: 'Comment',
          hintText: 'Optional description for this application',
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
      await client.createApplication(
        widget.tenantId,
        comment: _controller.text.isNotEmpty ? _controller.text : null,
      );
      ref.invalidate(applicationListProvider);
      if (mounted) Navigator.of(context).pop();
    } catch (e) {
      if (mounted) {
        setState(() => _loading = false);
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Failed to create application: $e')),
        );
      }
    }
  }
}
