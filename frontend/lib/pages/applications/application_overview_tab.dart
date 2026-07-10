import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:floating_snackbar/floating_snackbar.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/application_providers.dart';
import '../../providers/oceaniam_client_provider.dart';

class ApplicationOverviewTab extends ConsumerStatefulWidget {
  final String tenantId;
  final String applicationId;

  const ApplicationOverviewTab({
    super.key,
    required this.tenantId,
    required this.applicationId,
  });

  @override
  ConsumerState<ApplicationOverviewTab> createState() =>
      _ApplicationOverviewTabState();
}

class _ApplicationOverviewTabState
    extends ConsumerState<ApplicationOverviewTab> {
  final _commentController = TextEditingController();
  String? _syncedComment;
  bool _saving = false;

  @override
  void dispose() {
    _commentController.dispose();
    super.dispose();
  }

  void _syncComment(ApplicationDetail detail) {
    final next = detail.comment ?? '';
    if (_syncedComment == next) return;
    _syncedComment = next;
    if (_commentController.text != next) {
      _commentController.text = next;
    }
  }

  String get _jwksUrl {
    final base = kBackendBaseUrl.replaceAll(RegExp(r'/+$'), '');
    return '$base/tenants/${widget.tenantId}/.well-known/jwks.json';
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final detailAsync = ref.watch(
      applicationDetailProvider(widget.tenantId, widget.applicationId),
    );

    return detailAsync.when(
      loading: () => const Padding(
        padding: EdgeInsets.symmetric(vertical: 24),
        child: Center(child: CircularProgressIndicator()),
      ),
      error: (e, _) => Padding(
        padding: const EdgeInsets.symmetric(vertical: 16),
        child: Text(
          'Failed to load application: $e',
          style: theme.textTheme.bodyMedium?.copyWith(
            color: theme.colorScheme.error,
          ),
        ),
      ),
      data: (detail) {
        _syncComment(detail);
        final canSave =
            !_saving && _commentController.text != (detail.comment ?? '');

        return Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            _InfoRow(label: 'Application ID', value: detail.id, mono: true),
            const SizedBox(height: 12),
            _InfoRow(label: 'Tenant ID', value: detail.tenantId, mono: true),
            const SizedBox(height: 16),
            Text(
              'Comment',
              style: theme.textTheme.labelMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 4),
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  child: TextField(
                    controller: _commentController,
                    enabled: !_saving,
                    minLines: 1,
                    maxLines: 3,
                    decoration: const InputDecoration(
                      hintText: 'Optional description',
                      isDense: true,
                    ),
                    onChanged: (_) => setState(() {}),
                  ),
                ),
                const SizedBox(width: 8),
                FilledButton(
                  onPressed: canSave ? _saveComment : null,
                  child: _saving
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('Save'),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Text(
              '.well-known JWKS',
              style: theme.textTheme.labelMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 4),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: theme.colorScheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(12),
              ),
              child: Row(
                children: [
                  Expanded(
                    child: SelectableText(
                      _jwksUrl,
                      style: theme.textTheme.bodySmall?.copyWith(
                        fontFamily: 'monospace',
                      ),
                    ),
                  ),
                  IconButton(
                    tooltip: 'Copy',
                    onPressed: () async {
                      await Clipboard.setData(ClipboardData(text: _jwksUrl));
                      if (context.mounted) {
                        FloatingSnackBar.success(context, 'JWKS URL copied');
                      }
                    },
                    icon: const Icon(FluentIcons.copy_24_regular),
                  ),
                ],
              ),
            ),
          ],
        );
      },
    );
  }

  Future<void> _saveComment() async {
    setState(() => _saving = true);
    try {
      final client = ref.read(oceanIAMClientProvider);
      final next = _commentController.text.trim();
      await client.updateApplication(
        widget.tenantId,
        widget.applicationId,
        comment: next.isEmpty ? null : next,
      );
      _syncedComment = next;
      ref.invalidate(
        applicationDetailProvider(widget.tenantId, widget.applicationId),
      );
      ref.invalidate(applicationListProvider);
      if (mounted) {
        FloatingSnackBar.success(context, 'Comment saved');
      }
    } catch (e) {
      if (mounted) {
        FloatingSnackBar.error(context, 'Failed to save comment: $e');
      }
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }
}

class _InfoRow extends StatelessWidget {
  final String label;
  final String value;
  final bool mono;

  const _InfoRow({required this.label, required this.value, this.mono = false});

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
