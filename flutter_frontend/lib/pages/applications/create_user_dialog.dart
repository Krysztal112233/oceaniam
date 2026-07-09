import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:floating_snackbar/floating_snackbar.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';
import 'package:word_generator/word_generator.dart';

import '../../providers/application_providers.dart';
import '../../providers/oceaniam_client_provider.dart';

enum _ContactMethod { email, phone }

class CreateUserDialog extends ConsumerStatefulWidget {
  final String tenantId;
  final String applicationId;

  const CreateUserDialog({
    super.key,
    required this.tenantId,
    required this.applicationId,
  });

  @override
  ConsumerState<CreateUserDialog> createState() => _CreateUserDialogState();
}

class _CreateUserDialogState extends ConsumerState<CreateUserDialog> {
  final _nicknameController = TextEditingController();
  final _passwordController = TextEditingController();
  final _emailController = TextEditingController();
  final _phoneController = TextEditingController();
  bool _loading = false;
  bool _obscurePassword = true;
  _ContactMethod _contactMethod = _ContactMethod.email;

  void _switchContactMethod(_ContactMethod method) {
    if (_contactMethod == method) return;
    if (method == _ContactMethod.phone) {
      _phoneController.text = _emailController.text;
    } else {
      _emailController.text = _phoneController.text;
    }
    setState(() => _contactMethod = method);
  }

  @override
  void dispose() {
    _nicknameController.dispose();
    _passwordController.dispose();
    _emailController.dispose();
    _phoneController.dispose();
    super.dispose();
  }

  static String _capitalize(String word) {
    if (word.isEmpty) return word;
    return word[0].toUpperCase() + word.substring(1);
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final canSubmit =
        _nicknameController.text.isNotEmpty &&
        _passwordController.text.isNotEmpty;
    final contactController = _contactMethod == _ContactMethod.email
        ? _emailController
        : _phoneController;
    final contactHint = _contactMethod == _ContactMethod.email
        ? 'Enter email address'
        : 'Enter phone number';
    final contactLabel = _contactMethod == _ContactMethod.email
        ? 'Email'
        : 'Phone';

    return AlertDialog(
      backgroundColor: theme.colorScheme.surfaceContainerHigh,
      surfaceTintColor: theme.colorScheme.surfaceTint,
      elevation: 3,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(28)),
      titlePadding: const EdgeInsets.fromLTRB(24, 24, 24, 0),
      contentPadding: const EdgeInsets.fromLTRB(24, 20, 24, 0),
      actionsPadding: const EdgeInsets.fromLTRB(24, 8, 24, 16),
      title: const Text('Create User'),
      content: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              controller: _nicknameController,
              autofocus: true,
              enabled: !_loading,
              decoration: InputDecoration(
                labelText: 'Nickname *',
                hintText: 'Enter a unique nickname',
                suffixIcon: IconButton(
                  icon: const Icon(Icons.casino_outlined),
                  tooltip: 'Generate random nickname',
                  onPressed: () {
                    final generator = WordGenerator();
                    final first = _capitalize(generator.randomNoun());
                    final second = _capitalize(generator.randomNoun());
                    _nicknameController.text = '${first}_$second';
                    setState(() {});
                  },
                ),
              ),
              onChanged: (_) => setState(() {}),
              onSubmitted: (_) => canSubmit && !_loading ? _create() : null,
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _passwordController,
              enabled: !_loading,
              obscureText: _obscurePassword,
              decoration: InputDecoration(
                labelText: 'Password *',
                hintText: 'Enter a secure password',
                suffixIcon: IconButton(
                  icon: Icon(
                    _obscurePassword ? Icons.visibility_off : Icons.visibility,
                  ),
                  onPressed: () =>
                      setState(() => _obscurePassword = !_obscurePassword),
                ),
              ),
              onChanged: (_) => setState(() {}),
              onSubmitted: (_) => canSubmit && !_loading ? _create() : null,
            ),
            const SizedBox(height: 12),
            TextField(
              controller: contactController,
              enabled: !_loading,
              decoration: InputDecoration(
                labelText: contactLabel,
                hintText: contactHint,
                suffixIcon: IconButton(
                  icon: Icon(
                    _contactMethod == _ContactMethod.email
                        ? Icons.email_outlined
                        : Icons.phone_outlined,
                  ),
                  tooltip: _contactMethod == _ContactMethod.email
                      ? 'Switch to phone'
                      : 'Switch to email',
                  onPressed: () => _switchContactMethod(
                    _contactMethod == _ContactMethod.email
                        ? _ContactMethod.phone
                        : _ContactMethod.email,
                  ),
                ),
              ),
              keyboardType: _contactMethod == _ContactMethod.email
                  ? TextInputType.emailAddress
                  : TextInputType.phone,
              onSubmitted: (_) => canSubmit && !_loading ? _create() : null,
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: _loading ? null : () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: canSubmit && !_loading ? _create : null,
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
      await client.createUser(
        widget.tenantId,
        widget.applicationId,
        CreateUserRequest(
          nickname: _nicknameController.text,
          password: _passwordController.text,
          email: _contactMethod == _ContactMethod.email
              ? _emailController.text.isNotEmpty
                    ? _emailController.text
                    : null
              : null,
          phone: _contactMethod == _ContactMethod.phone
              ? _phoneController.text.isNotEmpty
                    ? _phoneController.text
                    : null
              : null,
        ),
      );
      ref.invalidate(applicationUsersPageProvider);
      if (mounted) Navigator.of(context).pop();
      if (mounted) {
        FloatingSnackBar.success(
          context,
          'User "${_nicknameController.text}" created',
        );
      }
    } catch (e) {
      if (mounted) {
        setState(() => _loading = false);
        FloatingSnackBar.error(context, 'Failed to create user: $e');
      }
    }
  }
}
