import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:floating_snackbar/floating_snackbar.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_table_view/material_table_view.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/key_providers.dart';
import '../../providers/oceaniam_client_provider.dart';
import '../../widgets/admin_page_scaffold.dart';
import '../../widgets/confirm_delete_dialog.dart';

/// Tenant signing keys: list / rotate / revoke + JWKS URL.
///
/// Backend:
/// - GET /tenants/{tid}/keys
/// - POST /tenants/{tid}/keys
/// - DELETE /tenants/{tid}/keys/{key_id}
/// - GET /tenants/{tid}/.well-known/jwks.json
class TenantKeysPage extends ConsumerStatefulWidget {
  final String tenantId;

  const TenantKeysPage({super.key, required this.tenantId});

  @override
  ConsumerState<TenantKeysPage> createState() => _TenantKeysPageState();
}

class _TenantKeysPageState extends ConsumerState<TenantKeysPage> {
  static const _rowHeight = kMinInteractiveDimension;
  static const _headerHeight = 48.0;

  static const _columns = [
    TableColumn(width: 160, flex: 3),
    TableColumn(width: 100, flex: 1),
    TableColumn(width: 100, flex: 1),
    TableColumn(width: 160, flex: 2),
    TableColumn(width: 160, flex: 2),
    TableColumn(width: 160, flex: 2),
    TableColumn(width: 88),
  ];

  static const _headers = [
    'Key ID',
    'Algorithm',
    'Status',
    'Created',
    'Activated',
    'Expires',
    '',
  ];

  List<ApplicationKey>? _lastKeys;
  ApplicationKey? _newlyCreatedKey;
  bool _rotating = false;
  String? _revokingKeyId;

  String get _jwksUrl {
    final base = kBackendBaseUrl.replaceAll(RegExp(r'/+$'), '');
    return '$base/tenants/${widget.tenantId}/.well-known/jwks.json';
  }

  @override
  void didUpdateWidget(covariant TenantKeysPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.tenantId != widget.tenantId) {
      _lastKeys = null;
      _newlyCreatedKey = null;
      _rotating = false;
      _revokingKeyId = null;
    }
  }

  @override
  Widget build(BuildContext context) {
    if (widget.tenantId.isEmpty) {
      return AdminPageScaffold(
        title: 'Signing keys',
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                FluentIcons.certificate_24_regular,
                size: 48,
                color: Theme.of(context).colorScheme.outline,
              ),
              const SizedBox(height: 16),
              Text(
                'Select a tenant to manage signing keys',
                style: Theme.of(context).textTheme.titleMedium,
              ),
            ],
          ),
        ),
      );
    }

    final async = ref.watch(tenantKeysProvider(widget.tenantId));
    ref.listen(tenantKeysProvider(widget.tenantId), (prev, next) {
      final data = next.valueOrNull;
      if (data != null && !identical(_lastKeys, data)) {
        setState(() => _lastKeys = data);
      }
    });
    final keys = async.valueOrNull ?? _lastKeys;
    final reloading = async.isLoading && keys != null;

    return AdminPageScaffold(
      title: 'Signing keys',
      actions: [
        FilledButton.icon(
          key: const Key('rotate-key'),
          onPressed: _rotating || async.isLoading ? null : _rotate,
          icon: _rotating
              ? const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Icon(FluentIcons.arrow_sync_24_regular),
          label: const Text('Rotate'),
        ),
      ],
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (reloading) const LinearProgressIndicator(minHeight: 2),
          Expanded(child: _buildBody(context, async, keys)),
        ],
      ),
    );
  }

  Widget _buildBody(
    BuildContext context,
    AsyncValue<List<ApplicationKey>> async,
    List<ApplicationKey>? keys,
  ) {
    if (keys == null) {
      if (async.isLoading) {
        return const Center(child: CircularProgressIndicator());
      }
      if (async.hasError) {
        return Center(child: Text('Failed to load keys: ${async.error}'));
      }
      return const SizedBox.shrink();
    }

    if (async.hasError && keys.isEmpty) {
      return Center(child: Text('Failed to load keys: ${async.error}'));
    }

    final theme = Theme.of(context);

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        _JwksCard(url: _jwksUrl),
        if (_newlyCreatedKey != null) ...[
          const SizedBox(height: 12),
          _NewKeyBanner(keyData: _newlyCreatedKey!),
        ],
        const SizedBox(height: 16),
        if (keys.isEmpty)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 48),
            child: Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    FluentIcons.certificate_24_regular,
                    size: 48,
                    color: theme.colorScheme.outline,
                  ),
                  const SizedBox(height: 16),
                  Text(
                    'No signing keys yet',
                    style: theme.textTheme.titleMedium,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Rotate to generate the first key pair.',
                    style: theme.textTheme.bodyMedium?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
                ],
              ),
            ),
          )
        else
          SizedBox(
            height: _headerHeight + keys.length * _rowHeight,
            child: TableView.builder(
              columns: _columns,
              rowCount: keys.length,
              rowHeight: _rowHeight,
              headerHeight: _headerHeight,
              headerBuilder: (context, contentBuilder) {
                return contentBuilder(context, (context, column) {
                  return _HeaderCell(label: _headers[column]);
                });
              },
              rowBuilder: (context, row, contentBuilder) {
                final key = keys[row];
                return contentBuilder(context, (context, column) {
                  return switch (column) {
                    0 => _CellText(key.keyId, mono: true),
                    1 => _CellText(key.algorithm),
                    2 => _StatusBadge(key.status),
                    3 => _CellText(_formatDate(key.createdAt)),
                    4 => _CellText(_formatDate(key.activatedAt)),
                    5 => _CellText(_formatDate(key.expiresAt)),
                    6 =>
                      key.status == 'Active'
                          ? Align(
                              alignment: Alignment.centerRight,
                              child: Padding(
                                padding: const EdgeInsets.only(right: 4),
                                child: TextButton(
                                  onPressed: _revokingKeyId != null
                                      ? null
                                      : () => _revoke(key),
                                  child: _revokingKeyId == key.keyId
                                      ? const SizedBox(
                                          width: 16,
                                          height: 16,
                                          child: CircularProgressIndicator(
                                            strokeWidth: 2,
                                          ),
                                        )
                                      : const Text('Revoke'),
                                ),
                              ),
                            )
                          : const SizedBox.shrink(),
                    _ => const SizedBox.shrink(),
                  };
                });
              },
            ),
          ),
      ],
    );
  }

  Future<void> _rotate() async {
    setState(() => _rotating = true);
    try {
      final client = ref.read(oceanIAMClientProvider);
      await client.rotateKey(widget.tenantId);
      ref.invalidate(tenantKeysProvider(widget.tenantId));
      final keys = await ref.read(tenantKeysProvider(widget.tenantId).future);
      ApplicationKey? latest;
      for (final k in keys.where((k) => k.status == 'Active')) {
        if (latest == null || k.activatedAt.compareTo(latest.activatedAt) > 0) {
          latest = k;
        }
      }
      if (mounted) {
        setState(() {
          _lastKeys = keys;
          _newlyCreatedKey = latest;
        });
        FloatingSnackBar.success(context, 'Key rotated');
      }
    } catch (e) {
      if (mounted) {
        FloatingSnackBar.error(context, 'Failed to rotate key: $e');
      }
    } finally {
      if (mounted) setState(() => _rotating = false);
    }
  }

  Future<void> _revoke(ApplicationKey key) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => ConfirmDeleteDialog(
        title: 'Revoke signing key',
        itemName: key.keyId,
        confirmButtonText: 'Revoke',
      ),
    );
    if (confirmed != true) return;

    setState(() => _revokingKeyId = key.keyId);
    try {
      final client = ref.read(oceanIAMClientProvider);
      await client.revokeKey(widget.tenantId, key.keyId);
      if (_newlyCreatedKey?.keyId == key.keyId) {
        _newlyCreatedKey = null;
      }
      ref.invalidate(tenantKeysProvider(widget.tenantId));
      if (mounted) {
        FloatingSnackBar.success(context, 'Key revoked');
      }
    } catch (e) {
      if (mounted) {
        FloatingSnackBar.error(context, 'Failed to revoke key: $e');
      }
    } finally {
      if (mounted) setState(() => _revokingKeyId = null);
    }
  }
}

String _formatDate(String raw) {
  final dt = DateTime.tryParse(raw);
  if (dt == null) return raw;
  final local = dt.toLocal();
  String two(int n) => n.toString().padLeft(2, '0');
  return '${local.year}-${two(local.month)}-${two(local.day)} '
      '${two(local.hour)}:${two(local.minute)}:${two(local.second)}';
}

class _JwksCard extends StatelessWidget {
  final String url;

  const _JwksCard({required this.url});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      clipBehavior: Clip.antiAlias,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '.well-known JWKS',
              style: theme.textTheme.labelMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 8),
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
                      url,
                      style: theme.textTheme.bodySmall?.copyWith(
                        fontFamily: 'monospace',
                      ),
                    ),
                  ),
                  IconButton(
                    tooltip: 'Copy',
                    onPressed: () async {
                      await Clipboard.setData(ClipboardData(text: url));
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
        ),
      ),
    );
  }
}

class _NewKeyBanner extends StatelessWidget {
  final ApplicationKey keyData;

  const _NewKeyBanner({required this.keyData});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      color: theme.colorScheme.primaryContainer,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Newly rotated key',
              style: theme.textTheme.titleSmall?.copyWith(
                color: theme.colorScheme.onPrimaryContainer,
              ),
            ),
            const SizedBox(height: 8),
            SelectableText(
              'Key ID: ${keyData.keyId}',
              style: theme.textTheme.bodyMedium?.copyWith(
                fontFamily: 'monospace',
                color: theme.colorScheme.onPrimaryContainer,
              ),
            ),
            SelectableText(
              'Algorithm: ${keyData.algorithm}',
              style: theme.textTheme.bodyMedium?.copyWith(
                fontFamily: 'monospace',
                color: theme.colorScheme.onPrimaryContainer,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _StatusBadge extends StatelessWidget {
  final String status;

  const _StatusBadge(this.status);

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final Color bg;
    final Color fg;
    switch (status) {
      case 'Active':
        bg = theme.colorScheme.primaryContainer;
        fg = theme.colorScheme.onPrimaryContainer;
      case 'Revoked':
        bg = theme.colorScheme.errorContainer;
        fg = theme.colorScheme.onErrorContainer;
      default:
        bg = theme.colorScheme.surfaceContainerHighest;
        fg = theme.colorScheme.onSurfaceVariant;
    }
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Align(
        alignment: Alignment.centerLeft,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            color: bg,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Text(
            status,
            overflow: TextOverflow.ellipsis,
            style: theme.textTheme.labelMedium?.copyWith(color: fg),
          ),
        ),
      ),
    );
  }
}

class _HeaderCell extends StatelessWidget {
  final String label;

  const _HeaderCell({required this.label});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Align(
        alignment: Alignment.centerLeft,
        child: Text(
          label,
          style: Theme.of(context).textTheme.labelLarge,
          overflow: TextOverflow.ellipsis,
        ),
      ),
    );
  }
}

class _CellText extends StatelessWidget {
  final String text;
  final bool mono;

  const _CellText(this.text, {this.mono = false});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Align(
        alignment: Alignment.centerLeft,
        child: Text(
          text,
          overflow: TextOverflow.ellipsis,
          style: mono
              ? Theme.of(
                  context,
                ).textTheme.bodyMedium?.copyWith(fontFamily: 'monospace')
              : null,
        ),
      ),
    );
  }
}
