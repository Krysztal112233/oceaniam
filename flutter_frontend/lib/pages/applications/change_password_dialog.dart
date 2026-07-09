import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:floating_snackbar/floating_snackbar.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/oceaniam_client_provider.dart';

class ChangePasswordDialog extends ConsumerStatefulWidget {
  final String tenantId;
  final String applicationId;
  final ApplicationUser user;

  const ChangePasswordDialog({
    super.key,
    required this.tenantId,
    required this.applicationId,
    required this.user,
  });

  @override
  ConsumerState<ChangePasswordDialog> createState() =>
      _ChangePasswordDialogState();
}

class _ChangePasswordDialogState extends ConsumerState<ChangePasswordDialog> {
  final _passwordController = TextEditingController();
  final _confirmController = TextEditingController();
  bool _loading = false;
  bool _obscurePassword = true;
  bool _obscureConfirm = true;
  String? _validationError;

  static const _minLength = 12;

  @override
  void dispose() {
    _passwordController.dispose();
    _confirmController.dispose();
    super.dispose();
  }

  bool get _canSubmit {
    final password = _passwordController.text;
    final confirm = _confirmController.text;
    return password.length >= _minLength && confirm.isNotEmpty && !_loading;
  }

  String? _validate() {
    final password = _passwordController.text;
    final confirm = _confirmController.text;
    if (password.length < _minLength) {
      return 'Password must be at least $_minLength characters';
    }
    if (password != confirm) {
      return 'Passwords do not match';
    }
    return null;
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
      title: const Text('Change Password'),
      content: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              'Set a new password for ${widget.user.nickname}.',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 16),
            TextField(
              controller: _passwordController,
              autofocus: true,
              enabled: !_loading,
              obscureText: _obscurePassword,
              decoration: InputDecoration(
                labelText: 'New password *',
                hintText: 'At least $_minLength characters',
                suffixIcon: IconButton(
                  icon: Icon(
                    _obscurePassword ? Icons.visibility_off : Icons.visibility,
                  ),
                  onPressed: () =>
                      setState(() => _obscurePassword = !_obscurePassword),
                ),
              ),
              onChanged: (_) {
                if (_validationError != null) {
                  setState(() => _validationError = null);
                } else {
                  setState(() {});
                }
              },
              onSubmitted: (_) {
                if (_canSubmit) _submit();
              },
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _confirmController,
              enabled: !_loading,
              obscureText: _obscureConfirm,
              decoration: InputDecoration(
                labelText: 'Confirm password *',
                hintText: 'Re-enter new password',
                suffixIcon: IconButton(
                  icon: Icon(
                    _obscureConfirm ? Icons.visibility_off : Icons.visibility,
                  ),
                  onPressed: () =>
                      setState(() => _obscureConfirm = !_obscureConfirm),
                ),
              ),
              onChanged: (_) {
                if (_validationError != null) {
                  setState(() => _validationError = null);
                } else {
                  setState(() {});
                }
              },
              onSubmitted: (_) {
                if (_canSubmit) _submit();
              },
            ),
            if (_validationError != null) ...[
              const SizedBox(height: 12),
              Text(
                _validationError!,
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.error,
                ),
              ),
            ],
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: _loading ? null : () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: _canSubmit ? _submit : null,
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
    final error = _validate();
    if (error != null) {
      setState(() => _validationError = error);
      return;
    }

    setState(() {
      _loading = true;
      _validationError = null;
    });

    try {
      final client = ref.read(oceanIAMClientProvider);
      await client.updateUserPassword(
        widget.tenantId,
        widget.applicationId,
        widget.user.id,
        _passwordController.text,
      );
      if (mounted) Navigator.of(context).pop(true);
      if (mounted) {
        FloatingSnackBar.success(
          context,
          'Password updated for ${widget.user.nickname}',
        );
      }
    } catch (e) {
      if (mounted) {
        setState(() => _loading = false);
        FloatingSnackBar.error(context, 'Failed to update password: $e');
      }
    }
  }
}
