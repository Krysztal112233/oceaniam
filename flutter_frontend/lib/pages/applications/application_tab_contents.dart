import 'package:data_table_2/data_table_2.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:floating_snackbar/floating_snackbar.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/application_providers.dart';
import '../../providers/oceaniam_client_provider.dart';
import '../../widgets/confirm_delete_dialog.dart';

class ApplicationOverviewTab extends StatelessWidget {
  final String applicationId;

  const ApplicationOverviewTab({super.key, required this.applicationId});

  @override
  Widget build(BuildContext context) {
    return _PlaceholderBody(
      icon: FluentIcons.info_24_regular,
      title: 'Overview',
      description: 'Application metadata and settings for $applicationId.',
    );
  }
}

class ApplicationUsersTab extends ConsumerWidget {
  final String tenantId;
  final String applicationId;
  final Widget? action;

  const ApplicationUsersTab({
    super.key,
    required this.tenantId,
    required this.applicationId,
    this.action,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final usersAsync = ref.watch(
      applicationUsersProvider(tenantId, applicationId),
    );
    final theme = Theme.of(context);

    return Padding(
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
              Text('Users', style: theme.textTheme.titleMedium),
              const Spacer(),
              action ?? const SizedBox.shrink(),
            ],
          ),
          const SizedBox(height: 16),
          Expanded(
            child: usersAsync.when(
              data: (users) {
                if (users.isEmpty) {
                  return _PlaceholderBody(
                    icon: FluentIcons.people_24_regular,
                    title: 'No users yet',
                    description: 'This application has no users.',
                  );
                }
                return _UsersTable(
                  users: users,
                  onView: (user) {
                    // TODO: navigate to user detail
                  },
                );
              },
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (error, stack) =>
                  Center(child: Text('Failed to load users: $error')),
            ),
          ),
        ],
      ),
    );
  }
}

class _UsersTable extends StatelessWidget {
  final List<ApplicationUser> users;
  final ValueChanged<ApplicationUser>? onView;

  const _UsersTable({required this.users, this.onView});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return DataTable2(
      columns: const [
        DataColumn2(label: Text('Nickname'), size: ColumnSize.L),
        DataColumn2(label: Text('Email')),
        DataColumn2(label: Text('Phone')),
        DataColumn2(label: Text('Actions'), size: ColumnSize.S),
      ],
      rows: users.map((user) {
        return DataRow2(
          cells: [
            DataCell(Text(user.nickname)),
            DataCell(Text(user.email ?? '-')),
            DataCell(Text(user.phone ?? '-')),
            DataCell(
              Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  IconButton(
                    icon: const Icon(FluentIcons.eye_24_regular, size: 20),
                    tooltip: 'View',
                    onPressed: () => onView?.call(user),
                  ),
                ],
              ),
            ),
          ],
        );
      }).toList(),
      headingTextStyle: theme.textTheme.titleSmall?.copyWith(
        fontWeight: FontWeight.bold,
      ),
      dataTextStyle: theme.textTheme.bodyMedium,
      minWidth: 600,
      empty: const _EmptyTableMessage(
        icon: FluentIcons.people_24_regular,
        message: 'No users found',
      ),
    );
  }
}

class _EmptyTableMessage extends StatelessWidget {
  final IconData icon;
  final String message;

  const _EmptyTableMessage({required this.icon, required this.message});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 40, color: theme.colorScheme.outline),
          const SizedBox(height: 12),
          Text(message, style: theme.textTheme.bodyMedium),
        ],
      ),
    );
  }
}

class ApplicationSecretsTab extends StatelessWidget {
  final String applicationId;

  const ApplicationSecretsTab({super.key, required this.applicationId});

  @override
  Widget build(BuildContext context) {
    return _PlaceholderBody(
      icon: FluentIcons.key_24_regular,
      title: 'Secrets',
      description: 'Manage secret bindings for $applicationId.',
    );
  }
}

class ApplicationSettingsTab extends ConsumerWidget {
  final String tenantId;
  final String applicationId;

  const ApplicationSettingsTab({
    super.key,
    required this.tenantId,
    required this.applicationId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Card(
            child: ListTile(
              leading: Icon(
                FluentIcons.delete_24_regular,
                color: theme.colorScheme.error,
              ),
              title: Text(
                'Delete application',
                style: TextStyle(color: theme.colorScheme.error),
              ),
              subtitle: Text('Remove "$applicationId" from this tenant.'),
              onTap: () => _confirmAndDelete(context, ref),
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _confirmAndDelete(BuildContext context, WidgetRef ref) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => const ConfirmDeleteDialog(
        title: 'Delete application',
        itemName: 'application',
        confirmButtonText: 'Delete',
      ),
    );
    if (confirmed != true) return;

    try {
      final client = ref.read(oceanIAMClientProvider);
      await client.deleteApplication(tenantId, applicationId);
      ref.invalidate(applicationListProvider);
      if (context.mounted) {
        Navigator.of(context).maybePop();
        FloatingSnackBar.success(context, 'Deleted "$applicationId"');
      }
    } catch (e) {
      if (context.mounted) {
        FloatingSnackBar.error(context, 'Failed to delete: $e');
      }
    }
  }
}

class _PlaceholderBody extends StatelessWidget {
  final IconData icon;
  final String title;
  final String description;

  const _PlaceholderBody({
    required this.icon,
    required this.title,
    required this.description,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Icon(icon, size: 20, color: theme.colorScheme.primary),
              const SizedBox(width: 8),
              Text(title, style: theme.textTheme.titleMedium),
              const Spacer(),
            ],
          ),
          const SizedBox(height: 16),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            decoration: BoxDecoration(
              color: theme.colorScheme.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Column(
              children: [
                Icon(icon, size: 40, color: theme.colorScheme.outline),
                const SizedBox(height: 12),
                Text(
                  description,
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
    );
  }
}
