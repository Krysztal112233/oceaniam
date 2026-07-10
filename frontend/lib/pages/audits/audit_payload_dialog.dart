import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import 'audit_format.dart';

Future<void> showAuditPayloadDialog(BuildContext context, AuditLog log) {
  return showDialog<void>(
    context: context,
    builder: (ctx) => _AuditPayloadDialog(log: log),
  );
}

class _AuditPayloadDialog extends StatelessWidget {
  final AuditLog log;

  const _AuditPayloadDialog({required this.log});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final pretty = const JsonEncoder.withIndent('  ').convert(log.payload);

    return AlertDialog(
      title: Text(log.auditType),
      content: SizedBox(
        width: 520,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              'ID: ${log.id}',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              'Created: ${formatAuditCreatedAt(log.createdAt)}',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 12),
            ConstrainedBox(
              constraints: const BoxConstraints(maxHeight: 360),
              child: SingleChildScrollView(
                child: SelectableText(
                  pretty,
                  style: theme.textTheme.bodySmall?.copyWith(
                    fontFamily: 'monospace',
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }
}
