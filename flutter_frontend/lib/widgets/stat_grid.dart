import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../providers/dashboard_providers.dart';
import 'stat_card.dart';

/// 平台概览统计卡片网格：5 张卡（租户 / 应用 / 用户 / 管理员 / 活跃密钥）。
///
/// 数据来自 [dashboardOverviewProvider]，不受趋势筛选影响。
/// 加载时显示骨架占位；出错时显示重试按钮。
class StatGrid extends ConsumerWidget {
  const StatGrid({super.key});

  static String _fmt(int n) {
    final s = n.toString();
    final out = StringBuffer();
    for (var i = 0; i < s.length; i++) {
      if (i > 0 && (s.length - i) % 3 == 0) out.write(',');
      out.write(s[i]);
    }
    return out.toString();
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final overview = ref.watch(dashboardOverviewProvider);
    final crossAxis = MediaQuery.of(context).size.width >= 600 ? 5 : 2;

    return overview.when(
      loading: () => _SkeletonGrid(crossAxisCount: crossAxis),
      error: (err, stack) => _ErrorBox(
        message: 'Failed to load overview: $err',
        onRetry: () => ref.invalidate(dashboardOverviewProvider),
      ),
      data: (o) =>
          _StatGridContent(overview: o, crossAxisCount: crossAxis, fmt: _fmt),
    );
  }
}

class _StatGridContent extends StatelessWidget {
  final Overview overview;
  final int crossAxisCount;
  final String Function(int) fmt;

  const _StatGridContent({
    required this.overview,
    required this.crossAxisCount,
    required this.fmt,
  });

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final cards = <StatCard>[
      StatCard(
        icon: FluentIcons.organization_24_regular,
        iconColor: scheme.primary,
        label: 'Tenants',
        value: fmt(overview.totalTenants),
        description: 'Total tenants on the platform',
      ),
      StatCard(
        icon: FluentIcons.apps_24_regular,
        iconColor: scheme.secondary,
        label: 'Applications',
        value: fmt(overview.totalApplications),
        description: 'Total applications across tenants',
      ),
      StatCard(
        icon: FluentIcons.people_24_regular,
        iconColor: scheme.tertiary,
        label: 'Users',
        value: fmt(overview.totalApplicationUsers),
        description: 'Application users total',
      ),
      StatCard(
        icon: FluentIcons.person_accounts_24_regular,
        iconColor: scheme.error,
        label: 'Administrators',
        value: fmt(overview.totalAdministrators),
        description: 'Platform administrators',
      ),
      StatCard(
        icon: FluentIcons.key_24_regular,
        iconColor: scheme.outline,
        label: 'Active Secrets',
        value: fmt(overview.totalActiveSecrets),
        description: 'Currently active secrets',
      ),
    ];

    return GridView.count(
      crossAxisCount: crossAxisCount,
      padding: const EdgeInsets.all(16),
      crossAxisSpacing: 16,
      mainAxisSpacing: 16,
      childAspectRatio: 1.3,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      children: cards,
    );
  }
}

class _SkeletonGrid extends StatelessWidget {
  final int crossAxisCount;
  const _SkeletonGrid({required this.crossAxisCount});

  @override
  Widget build(BuildContext context) {
    return GridView.count(
      crossAxisCount: crossAxisCount,
      padding: const EdgeInsets.all(16),
      crossAxisSpacing: 16,
      mainAxisSpacing: 16,
      childAspectRatio: 1.3,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      children: List.generate(crossAxisCount == 5 ? 5 : 6, (_) {
        return Card(
          elevation: 0,
          shape: RoundedRectangleBorder(
            side: BorderSide(color: Theme.of(context).dividerColor),
            borderRadius: BorderRadius.circular(12),
          ),
          child: const Padding(
            padding: EdgeInsets.all(16),
            child: Center(
              child: SizedBox(
                width: 24,
                height: 24,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
            ),
          ),
        );
      }),
    );
  }
}

class _ErrorBox extends StatelessWidget {
  final String message;
  final VoidCallback onRetry;
  const _ErrorBox({required this.message, required this.onRetry});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Card(
        elevation: 0,
        shape: RoundedRectangleBorder(
          side: BorderSide(color: Theme.of(context).dividerColor),
          borderRadius: BorderRadius.circular(12),
        ),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Icon(
                Icons.error_outline,
                color: Theme.of(context).colorScheme.error,
              ),
              const SizedBox(width: 12),
              Expanded(child: Text(message)),
              TextButton(onPressed: onRetry, child: const Text('Retry')),
            ],
          ),
        ),
      ),
    );
  }
}
