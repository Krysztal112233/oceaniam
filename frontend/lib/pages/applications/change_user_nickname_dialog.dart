import 'package:floating_snackbar/floating_snackbar.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/oceaniam_client_provider.dart';

class ChangeUserNicknameDialog extends ConsumerStatefulWidget {
  final String tenantId;
  final String applicationId;
  final ApplicationUser user;

  const ChangeUserNicknameDialog({
    super.key,
    required this.tenantId,
    required this.applicationId,
    required this.user,
  });

  @override
  ConsumerState<ChangeUserNicknameDialog> createState() =>
      _ChangeUserNicknameDialogState();
}

class _ChangeUserNicknameDialogState
    extends ConsumerState<ChangeUserNicknameDialog> {
  late final TextEditingController _nicknameController;
  bool _loading = false;
  String? _validationError;

  @override
  void initState() {
    super.initState();
    _nicknameController = TextEditingController(text: widget.user.nickname);
  }

  @override
  void dispose() {
    _nicknameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final nickname = _nicknameController.text.trim();

    return AlertDialog(
      backgroundColor: theme.colorScheme.surfaceContainerHigh,
      surfaceTintColor: theme.colorScheme.surfaceTint,
      elevation: 3,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(28)),
      titlePadding: const EdgeInsets.fromLTRB(24, 24, 24, 0),
      contentPadding: const EdgeInsets.fromLTRB(24, 20, 24, 0),
      actionsPadding: const EdgeInsets.fromLTRB(24, 8, 24, 16),
      title: const Text('Change Nickname'),
      content: SizedBox(
        width: 360,
        child: TextField(
          controller: _nicknameController,
          autofocus: true,
          enabled: !_loading,
          decoration: InputDecoration(
            labelText: 'Nickname *',
            errorText: _validationError,
          ),
          onChanged: (_) {
            if (_validationError != null) {
              setState(() => _validationError = null);
            } else {
              setState(() {});
            }
          },
          onSubmitted: (_) {
            if (nickname.isNotEmpty && !_loading) _submit();
          },
        ),
      ),
      actions: [
        TextButton(
          onPressed: _loading ? null : () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: nickname.isNotEmpty && !_loading ? _submit : null,
          child: _loading
              ? const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Text('Save'),
        ),
      ],
    );
  }

  Future<void> _submit() async {
    final nickname = _nicknameController.text.trim();
    if (nickname.isEmpty) {
      setState(() => _validationError = 'Nickname cannot be empty');
      return;
    }

    setState(() {
      _loading = true;
      _validationError = null;
    });

    try {
      final user = await ref
          .read(oceanIAMClientProvider)
          .patchUser(
            widget.tenantId,
            widget.applicationId,
            widget.user.id,
            PatchUserRequest(nickname: nickname),
          );
      if (mounted) Navigator.of(context).pop(user);
    } catch (e) {
      if (mounted) {
        setState(() => _loading = false);
        FloatingSnackBar.error(context, 'Failed to update nickname: $e');
      }
    }
  }
}
