import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:floating_snackbar/floating_snackbar.dart';

import '../../providers/application_providers.dart';
import '../../providers/oceaniam_client_provider.dart';
import 'configuration/application_configuration_editor.dart';

class ApplicationConfigurationTab extends ConsumerWidget {
  final String tenantId;
  final String applicationId;

  const ApplicationConfigurationTab({
    super.key,
    required this.tenantId,
    required this.applicationId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final configuration = ref.watch(
      applicationConfigurationProvider(tenantId, applicationId),
    );

    return configuration.when(
      loading: () => const SizedBox(
        height: 240,
        child: Center(child: CircularProgressIndicator()),
      ),
      error: (error, _) => _ConfigurationLoadError(
        error: error,
        onRetry: () => ref.invalidate(
          applicationConfigurationProvider(tenantId, applicationId),
        ),
      ),
      data: (value) => ApplicationConfigurationEditor(
        key: ValueKey(value),
        configuration: value,
        onSave: (patch) async {
          try {
            final client = ref.read(oceanIAMClientProvider);
            await client.updateApplicationConfiguration(
              tenantId,
              applicationId,
              patch,
            );
            ref.invalidate(
              applicationConfigurationProvider(tenantId, applicationId),
            );
            if (context.mounted) {
              FloatingSnackBar.success(context, 'Configuration saved');
            }
            return true;
          } catch (error) {
            if (context.mounted) {
              FloatingSnackBar.error(
                context,
                'Failed to save configuration: $error',
              );
            }
            return false;
          }
        },
      ),
    );
  }
}

class _ConfigurationLoadError extends StatelessWidget {
  final Object error;
  final VoidCallback onRetry;

  const _ConfigurationLoadError({required this.error, required this.onRetry});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      elevation: 0,
      color: theme.colorScheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              'Failed to load application configuration.',
              style: theme.textTheme.titleMedium?.copyWith(
                color: theme.colorScheme.onErrorContainer,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              '$error',
              textAlign: TextAlign.center,
              style: TextStyle(color: theme.colorScheme.onErrorContainer),
            ),
            const SizedBox(height: 12),
            FilledButton.tonal(onPressed: onRetry, child: const Text('Retry')),
          ],
        ),
      ),
    );
  }
}
