import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:floating_snackbar/floating_snackbar.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/application_providers.dart';
import '../../providers/oceaniam_client_provider.dart';
import '../../providers/secret_providers.dart';
import '../../providers/tenant_providers.dart';
import '../../widgets/segmented_expand_panel.dart';

/// Inline detail for a secret row (expand-in-place, same pattern as applications).
class SecretExpandedPanel extends StatelessWidget {
  final Secret secret;

  const SecretExpandedPanel({super.key, required this.secret});

  @override
  Widget build(BuildContext context) {
    return SegmentedExpandPanel(
      initialIndex: 1,
      tabs: [
        ExpandPanelTab(
          icon: FluentIcons.info_24_regular,
          label: 'Overview',
          builder: (_) => _SecretOverviewTab(secret: secret),
        ),
        ExpandPanelTab(
          icon: FluentIcons.app_folder_24_regular,
          label: 'Bindings',
          builder: (_) => _SecretBindingsTab(secret: secret),
        ),
        ExpandPanelTab(
          icon: FluentIcons.settings_24_regular,
          label: 'Settings',
          builder: (_) => _SecretSettingsTab(secret: secret),
        ),
      ],
    );
  }
}

class _SecretOverviewTab extends StatelessWidget {
  final Secret secret;

  const _SecretOverviewTab({required this.secret});

  @override
  Widget build(BuildContext context) {
    final revoked = secret.revokedAt != null;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _InfoRow(label: 'ID', value: secret.id),
        const SizedBox(height: 12),
        _InfoRow(label: 'Secret', value: secret.secret ?? '—', mono: true),
        const SizedBox(height: 12),
        _InfoRow(label: 'Created', value: secret.createdAt),
        if (revoked) ...[
          const SizedBox(height: 12),
          _InfoRow(label: 'Revoked', value: secret.revokedAt!),
        ],
      ],
    );
  }
}

class _SecretBindingsTab extends ConsumerStatefulWidget {
  final Secret secret;

  const _SecretBindingsTab({required this.secret});

  @override
  ConsumerState<_SecretBindingsTab> createState() => _SecretBindingsTabState();
}

class _SecretBindingsTabState extends ConsumerState<_SecretBindingsTab> {
  String? _busyApplicationId;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final tenantId = ref.watch(currentTenantIdProvider);
    final appsAsync = ref.watch(applicationListProvider);
    final boundIds = widget.secret.applicationIds.toSet();
    final apps = appsAsync.valueOrNull ?? const <Application>[];
    final appById = {for (final a in apps) a.id: a};
    final unbound = apps.where((a) => !boundIds.contains(a.id)).toList();

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                'Bound applications',
                style: theme.textTheme.titleSmall,
              ),
            ),
            FilledButton.tonalIcon(
              onPressed: tenantId == null || unbound.isEmpty
                  ? null
                  : () => _showBindDialog(unbound),
              icon: const Icon(FluentIcons.add_24_regular, size: 18),
              label: const Text('Bind'),
            ),
          ],
        ),
        const SizedBox(height: 8),
        if (tenantId == null)
          Text(
            'Select a tenant to bind applications.',
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          )
        else if (widget.secret.applicationIds.isEmpty)
          Text(
            'Not bound to any application.',
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          )
        else
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: widget.secret.applicationIds.map((id) {
              final comment = appById[id]?.comment;
              final label = (comment != null && comment.isNotEmpty)
                  ? '$id · $comment'
                  : id;
              final busy = _busyApplicationId == id;
              return InputChip(
                avatar: busy
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : Icon(
                        FluentIcons.app_folder_24_regular,
                        size: 16,
                        color: theme.colorScheme.onSecondaryContainer,
                      ),
                label: Text(label),
                onDeleted: busy ? null : () => _unbind(id),
                deleteIcon: const Icon(FluentIcons.dismiss_24_regular, size: 16),
                deleteButtonTooltipMessage: 'Unbind',
              );
            }).toList(),
          ),
        if (tenantId != null && unbound.isEmpty && apps.isNotEmpty) ...[
          const SizedBox(height: 8),
          Text(
            'All applications in this tenant are already bound.',
            style: theme.textTheme.bodySmall?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
        ],
        if (appsAsync.hasError) ...[
          const SizedBox(height: 8),
          Text(
            'Failed to load applications: ${appsAsync.error}',
            style: theme.textTheme.bodySmall?.copyWith(
              color: theme.colorScheme.error,
            ),
          ),
        ],
      ],
    );
  }

  Future<void> _showBindDialog(List<Application> unbound) async {
    final selected = await showDialog<String>(
      context: context,
      builder: (ctx) => _BindApplicationDialog(applications: unbound),
    );
    if (selected == null || !mounted) return;
    await _bind(selected);
  }

  Future<void> _bind(String applicationId) async {
    setState(() => _busyApplicationId = applicationId);
    try {
      final client = ref.read(oceanIAMClientProvider);
      await client.bindSecret(widget.secret.id, applicationId);
      ref.invalidate(secretsPageProvider);
      if (mounted) {
        FloatingSnackBar.success(context, 'Bound to $applicationId');
      }
    } catch (e) {
      if (mounted) {
        FloatingSnackBar.error(context, 'Failed to bind: $e');
      }
    } finally {
      if (mounted) setState(() => _busyApplicationId = null);
    }
  }

  Future<void> _unbind(String applicationId) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Unbind application'),
        content: Text(
          'Remove binding between this secret and "$applicationId"?',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: const Text('Unbind'),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;

    setState(() => _busyApplicationId = applicationId);
    try {
      final client = ref.read(oceanIAMClientProvider);
      await client.unbindSecret(widget.secret.id, applicationId);
      ref.invalidate(secretsPageProvider);
      if (mounted) {
        FloatingSnackBar.success(context, 'Unbound $applicationId');
      }
    } catch (e) {
      if (mounted) {
        FloatingSnackBar.error(context, 'Failed to unbind: $e');
      }
    } finally {
      if (mounted) setState(() => _busyApplicationId = null);
    }
  }
}

class _BindApplicationDialog extends StatelessWidget {
  final List<Application> applications;

  const _BindApplicationDialog({required this.applications});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return AlertDialog(
      backgroundColor: theme.colorScheme.surfaceContainerHigh,
      surfaceTintColor: theme.colorScheme.surfaceTint,
      elevation: 3,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(28)),
      titlePadding: const EdgeInsets.fromLTRB(24, 24, 24, 0),
      contentPadding: const EdgeInsets.fromLTRB(8, 12, 8, 0),
      actionsPadding: const EdgeInsets.fromLTRB(24, 8, 24, 16),
      title: const Text('Bind application'),
      content: SizedBox(
        width: 360,
        child: applications.isEmpty
            ? const Padding(
                padding: EdgeInsets.all(16),
                child: Text('No applications available to bind.'),
              )
            : ListView.builder(
                shrinkWrap: true,
                itemCount: applications.length,
                itemBuilder: (context, i) {
                  final app = applications[i];
                  final subtitle =
                      app.comment != null && app.comment!.isNotEmpty
                      ? app.comment!
                      : null;
                  return ListTile(
                    leading: Icon(
                      FluentIcons.app_folder_24_regular,
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                    title: Text(app.id),
                    subtitle: subtitle != null ? Text(subtitle) : null,
                    onTap: () => Navigator.of(context).pop(app.id),
                  );
                },
              ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
      ],
    );
  }
}

class _SecretSettingsTab extends StatelessWidget {
  final Secret secret;

  const _SecretSettingsTab({required this.secret});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Danger zone',
          style: theme.textTheme.titleSmall?.copyWith(
            color: theme.colorScheme.error,
          ),
        ),
        const SizedBox(height: 8),
        Text(
          'Delete permanently revokes this secret.',
          style: theme.textTheme.bodyMedium?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
        const SizedBox(height: 12),
        OutlinedButton.icon(
          onPressed: null,
          icon: const Icon(FluentIcons.delete_24_regular),
          label: const Text('Delete secret'),
          style: OutlinedButton.styleFrom(
            foregroundColor: theme.colorScheme.error,
            side: BorderSide(
              color: theme.colorScheme.error.withValues(alpha: 0.5),
            ),
          ),
        ),
      ],
    );
  }
}

class _InfoRow extends StatelessWidget {
  final String label;
  final String value;
  final bool mono;

  const _InfoRow({
    required this.label,
    required this.value,
    this.mono = false,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: theme.textTheme.labelMedium?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
        const SizedBox(height: 4),
        SelectableText(
          value,
          style: theme.textTheme.bodyMedium?.copyWith(
            fontFamily: mono ? 'monospace' : null,
          ),
        ),
      ],
    );
  }
}
