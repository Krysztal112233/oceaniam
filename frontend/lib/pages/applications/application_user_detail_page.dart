import 'package:floating_snackbar/floating_snackbar.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/oceaniam_client_provider.dart';
import '../../widgets/admin_page_scaffold.dart';
import '../../widgets/confirm_delete_dialog.dart';
import 'change_password_dialog.dart';
import 'change_user_nickname_dialog.dart';

class ApplicationUserDetailPage extends ConsumerStatefulWidget {
  final String tenantId;
  final String applicationId;
  final ApplicationUser user;

  const ApplicationUserDetailPage({
    super.key,
    required this.tenantId,
    required this.applicationId,
    required this.user,
  });

  @override
  ConsumerState<ApplicationUserDetailPage> createState() =>
      _ApplicationUserDetailPageState();
}

class _ApplicationUserDetailPageState
    extends ConsumerState<ApplicationUserDetailPage> {
  late ApplicationUser _user;
  bool _deleting = false;

  @override
  void initState() {
    super.initState();
    _user = widget.user;
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Material(
      type: MaterialType.transparency,
      child: AdminPageScaffold(
        title: _user.nickname,
        leading: [
          IconButton(
            icon: const Icon(FluentIcons.arrow_left_24_regular),
            tooltip: 'Back',
            onPressed: _deleting
                ? null
                : () => Navigator.of(context).maybePop(),
          ),
        ],
        actions: [
          MenuAnchor(
            menuChildren: [
              MenuItemButton(
                onPressed: _deleting ? null : _confirmDelete,
                leadingIcon: const Icon(FluentIcons.delete_24_regular),
                child: const Text('Delete user'),
              ),
            ],
            builder: (context, controller, child) => FilledButton.tonalIcon(
              onPressed: _deleting
                  ? null
                  : () => controller.isOpen
                        ? controller.close()
                        : controller.open(),
              icon: const Icon(FluentIcons.more_horizontal_24_regular),
              label: const Text('More'),
            ),
          ),
        ],
        child: ListView(
          padding: const EdgeInsets.all(24),
          children: [
            Text('User ID', style: theme.textTheme.labelLarge),
            const SizedBox(height: 4),
            SelectableText(_user.id, style: theme.textTheme.bodyMedium),
            const SizedBox(height: 24),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(20),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('User status', style: theme.textTheme.titleMedium),
                    const SizedBox(height: 8),
                    Text(
                      'Status management will be available here.',
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(20),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Text('Account', style: theme.textTheme.titleMedium),
                    const SizedBox(height: 16),
                    OutlinedButton.icon(
                      onPressed: _deleting ? null : _changeNickname,
                      icon: const Icon(FluentIcons.edit_24_regular),
                      label: const Text('Change nickname'),
                    ),
                    const SizedBox(height: 12),
                    OutlinedButton.icon(
                      onPressed: _deleting ? null : _changePassword,
                      icon: const Icon(FluentIcons.key_24_regular),
                      label: const Text('Change password'),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _changeNickname() async {
    final user = await showDialog<ApplicationUser>(
      context: context,
      builder: (context) => ChangeUserNicknameDialog(
        tenantId: widget.tenantId,
        applicationId: widget.applicationId,
        user: _user,
      ),
    );
    if (user != null && mounted) setState(() => _user = user);
  }

  Future<void> _changePassword() => showDialog<bool>(
    context: context,
    builder: (context) => ChangePasswordDialog(
      tenantId: widget.tenantId,
      applicationId: widget.applicationId,
      user: _user,
    ),
  );

  Future<void> _confirmDelete() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => ConfirmDeleteDialog(
        title: 'Delete User',
        itemName: _user.nickname,
        confirmButtonText: 'Delete user',
      ),
    );
    if (confirmed == true && mounted) await _deleteUser();
  }

  Future<void> _deleteUser() async {
    setState(() => _deleting = true);
    try {
      await ref
          .read(oceanIAMClientProvider)
          .deleteUser(widget.tenantId, widget.applicationId, _user.id);
      if (mounted) Navigator.of(context).pop(true);
    } catch (e) {
      if (mounted) {
        setState(() => _deleting = false);
        FloatingSnackBar.error(context, 'Failed to delete user: $e');
      }
    }
  }
}
