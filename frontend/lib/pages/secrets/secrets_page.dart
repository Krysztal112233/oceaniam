import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/secret_providers.dart';
import '../../widgets/admin_page_scaffold.dart';
import '../../widgets/empty_state_illustration.dart';
import 'create_secret_dialog.dart';
import 'secret_card.dart';

/// 平台级 API Secrets 列表 + 创建（一次性明文）。
///
/// 后端：
/// - GET /secrets（SecretRead）
/// - POST /secrets（SecretCreate，返回一次性明文）
class SecretsPage extends ConsumerStatefulWidget {
  const SecretsPage({super.key});

  @override
  ConsumerState<SecretsPage> createState() => _SecretsPageState();
}

class _SecretsPageState extends ConsumerState<SecretsPage> {
  int _page = 1;
  PagedResponse<Secret>? _lastResponse;
  String? _expandedSecretId;

  @override
  Widget build(BuildContext context) {
    final async = ref.watch(secretsPageProvider(_page));
    ref.listen(secretsPageProvider(_page), (prev, next) {
      final data = next.valueOrNull;
      if (data != null && !identical(_lastResponse, data)) {
        setState(() => _lastResponse = data);
      }
    });
    final response = async.valueOrNull ?? _lastResponse;
    final reloading = async.isLoading && response != null;

    return AdminPageScaffold(
      title: 'API secrets',
      actions: [
        FilledButton.icon(
          key: const Key('create-secret'),
          onPressed: () => _showCreateDialog(context),
          icon: const Icon(FluentIcons.add_24_regular),
          label: const Text('New secret'),
        ),
      ],
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (reloading) const LinearProgressIndicator(minHeight: 2),
          Expanded(child: _buildBody(context, async, response)),
        ],
      ),
    );
  }

  Widget _buildBody(
    BuildContext context,
    AsyncValue<PagedResponse<Secret>> async,
    PagedResponse<Secret>? response,
  ) {
    if (response == null) {
      if (async.isLoading) {
        return const Center(child: CircularProgressIndicator());
      }
      if (async.hasError) {
        return Center(child: Text('Failed to load secrets: ${async.error}'));
      }
      return const SizedBox.shrink();
    }

    if (async.hasError && response.items.isEmpty) {
      return Center(child: Text('Failed to load secrets: ${async.error}'));
    }

    if (response.items.isEmpty) {
      return Center(
        child: EmptyStateIllustration(
          icon: FluentIcons.key_24_regular,
          title: 'No secrets yet',
          action: FilledButton.tonalIcon(
            onPressed: () => _showCreateDialog(context),
            icon: const Icon(FluentIcons.add_24_regular),
            label: const Text('Create your first secret'),
          ),
        ),
      );
    }

    final hasNext = response.pageInfo.hasNext;
    final total = response.pageInfo.total;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(
          child: ListView.builder(
            padding: const EdgeInsets.all(16),
            itemCount: response.items.length,
            itemBuilder: (context, i) {
              final secret = response.items[i];
              return SecretCard(
                secret: secret,
                isExpanded: _expandedSecretId == secret.id,
                onExpand: () => setState(() {
                  _expandedSecretId = _expandedSecretId == secret.id
                      ? null
                      : secret.id;
                }),
              );
            },
          ),
        ),
        _PaginationBar(
          page: _page,
          total: total,
          hasNext: hasNext,
          onPrev: _page > 1
              ? () => setState(() {
                  _page -= 1;
                  _lastResponse = null;
                  _expandedSecretId = null;
                })
              : null,
          onNext: hasNext
              ? () => setState(() {
                  _page += 1;
                  _lastResponse = null;
                  _expandedSecretId = null;
                })
              : null,
        ),
      ],
    );
  }

  Future<void> _showCreateDialog(BuildContext context) async {
    final id = await showDialog<String>(
      context: context,
      barrierDismissible: false,
      builder: (_) => const CreateSecretDialog(),
    );
    if (id != null && mounted) {
      setState(() {
        _page = 1;
        _lastResponse = null;
        _expandedSecretId = id;
      });
    }
  }
}

class _PaginationBar extends StatelessWidget {
  final int page;
  final int total;
  final bool hasNext;
  final VoidCallback? onPrev;
  final VoidCallback? onNext;

  const _PaginationBar({
    required this.page,
    required this.total,
    required this.hasNext,
    this.onPrev,
    this.onNext,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 12),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          Text(
            'Page $page · $total total',
            style: theme.textTheme.bodySmall?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
          IconButton(
            tooltip: 'Previous page',
            onPressed: onPrev,
            icon: const Icon(FluentIcons.chevron_left_24_regular),
          ),
          IconButton(
            tooltip: 'Next page',
            onPressed: onNext,
            icon: const Icon(FluentIcons.chevron_right_24_regular),
          ),
        ],
      ),
    );
  }
}
