import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:floating_snackbar/floating_snackbar.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/oceaniam_client_provider.dart';
import '../../providers/secret_providers.dart';

/// Creates a secret and shows the plaintext once. Returns the created id on success.
class CreateSecretDialog extends ConsumerStatefulWidget {
  const CreateSecretDialog({super.key});

  @override
  ConsumerState<CreateSecretDialog> createState() => _CreateSecretDialogState();
}

class _CreateSecretDialogState extends ConsumerState<CreateSecretDialog> {
  bool _loading = false;
  CreateSecretResponse? _created;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final created = _created;

    return AlertDialog(
      backgroundColor: theme.colorScheme.surfaceContainerHigh,
      surfaceTintColor: theme.colorScheme.surfaceTint,
      elevation: 3,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(28)),
      titlePadding: const EdgeInsets.fromLTRB(24, 24, 24, 0),
      contentPadding: const EdgeInsets.fromLTRB(24, 20, 24, 0),
      actionsPadding: const EdgeInsets.fromLTRB(24, 8, 24, 16),
      title: Text(created == null ? 'Create secret' : 'Secret created'),
      content: created == null
          ? const Text(
              'A new platform API secret will be generated. '
              'The full value is shown only once — copy it before closing.',
            )
          : _CreatedBody(created: created),
      actions: created == null
          ? [
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
            ]
          : [
              FilledButton(
                onPressed: () => Navigator.of(context).pop(created.id),
                child: const Text('Done'),
              ),
            ],
    );
  }

  Future<void> _create() async {
    setState(() => _loading = true);
    try {
      final client = ref.read(oceanIAMClientProvider);
      final result = await client.createSecret();
      ref.invalidate(secretsPageProvider);
      if (mounted) setState(() => _created = result);
    } catch (e) {
      if (mounted) {
        setState(() => _loading = false);
        FloatingSnackBar.error(context, 'Failed to create secret: $e');
      }
    }
  }
}

class _CreatedBody extends StatelessWidget {
  final CreateSecretResponse created;

  const _CreatedBody({required this.created});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          'Copy this secret now. It will not be shown again.',
          style: theme.textTheme.bodyMedium?.copyWith(
            color: theme.colorScheme.error,
            fontWeight: FontWeight.w500,
          ),
        ),
        const SizedBox(height: 16),
        Text('ID', style: theme.textTheme.labelMedium),
        const SizedBox(height: 4),
        SelectableText(created.id, style: theme.textTheme.bodyMedium),
        const SizedBox(height: 12),
        Text('Secret', style: theme.textTheme.labelMedium),
        const SizedBox(height: 4),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          decoration: BoxDecoration(
            color: theme.colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(12),
          ),
          child: Row(
            children: [
              Expanded(
                child: SelectableText(
                  created.secret,
                  style: theme.textTheme.bodyMedium?.copyWith(
                    fontFamily: 'monospace',
                  ),
                ),
              ),
              IconButton(
                tooltip: 'Copy',
                onPressed: () async {
                  await Clipboard.setData(ClipboardData(text: created.secret));
                  if (context.mounted) {
                    FloatingSnackBar.success(context, 'Secret copied');
                  }
                },
                icon: const Icon(FluentIcons.copy_24_regular),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
