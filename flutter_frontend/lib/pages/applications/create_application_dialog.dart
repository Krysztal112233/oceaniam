import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../providers/oceaniam_client_provider.dart';
import '../../providers/application_providers.dart';

class CreateApplicationDialog extends ConsumerStatefulWidget {
  final String tenantId;

  const CreateApplicationDialog({super.key, required this.tenantId});

  @override
  ConsumerState<CreateApplicationDialog> createState() =>
      _CreateApplicationDialogState();
}

class _CreateApplicationDialogState
    extends ConsumerState<CreateApplicationDialog> {
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
