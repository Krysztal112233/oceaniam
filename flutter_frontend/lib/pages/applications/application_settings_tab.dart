import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:floating_snackbar/floating_snackbar.dart';

import '../../providers/application_providers.dart';
import '../../providers/oceaniam_client_provider.dart';
import '../../widgets/confirm_delete_dialog.dart';

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
            clipBehavior: Clip.antiAlias,
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
