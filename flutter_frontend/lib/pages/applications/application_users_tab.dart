import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_table_view/material_table_view.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/application_providers.dart';

class ApplicationUsersTab extends ConsumerStatefulWidget {
  final String tenantId;
  final String applicationId;
  final Widget? action;

  /// When true (default), the table expands to fill remaining parent height.
  /// When false, height follows table content (e.g. expanded card panel).
  final bool fillAvailable;

  const ApplicationUsersTab({
    super.key,
    required this.tenantId,
    required this.applicationId,
    this.action,
    this.fillAvailable = true,
  });

  @override
  ConsumerState<ApplicationUsersTab> createState() =>
      _ApplicationUsersTabState();
}

class _ApplicationUsersTabState extends ConsumerState<ApplicationUsersTab> {
  static const _rowHeight = kMinInteractiveDimension;
  static const _headerHeight = 48.0;
  static const _pageSize = 10;
  static const _emptyBodyHeight = 160.0;

  static const _columns = [
    TableColumn(width: 180, flex: 2),
    TableColumn(width: 220, flex: 3),
    TableColumn(width: 140, flex: 2),
    TableColumn(width: 72),
  ];

  static const _headers = ['Nickname', 'Email', 'Phone', 'Actions'];

  int _page = 1;
  PagedResponse<ApplicationUser>? _lastResponse;

  double get _stableBodyHeight => _headerHeight + _pageSize * _rowHeight;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final usersAsync = ref.watch(
      applicationUsersPageProvider(
        widget.tenantId,
        widget.applicationId,
        _page,
      ),
    );

    ref.listen(
      applicationUsersPageProvider(
        widget.tenantId,
        widget.applicationId,
        _page,
      ),
      (previous, next) {
        final data = next.valueOrNull;
        if (data != null && !identical(_lastResponse, data)) {
          setState(() => _lastResponse = data);
        }
      },
    );

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        mainAxisSize: widget.fillAvailable
            ? MainAxisSize.max
            : MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Icon(
                FluentIcons.people_24_regular,
                size: 20,
                color: theme.colorScheme.primary,
              ),
              const SizedBox(width: 8),
              Text('Users', style: theme.textTheme.titleMedium),
              const Spacer(),
              widget.action ?? const SizedBox.shrink(),
            ],
          ),
          const SizedBox(height: 16),
          if (widget.fillAvailable)
            Expanded(child: _buildBody(theme, usersAsync))
          else
            _buildBody(theme, usersAsync),
        ],
      ),
    );
  }

  Widget _buildBody(
    ThemeData theme,
    AsyncValue<PagedResponse<ApplicationUser>> usersAsync,
  ) {
    final response = usersAsync.valueOrNull ?? _lastResponse;
    final isLoading = usersAsync.isLoading;
    final error = usersAsync.hasError && response == null
        ? usersAsync.error
        : null;

    if (error != null) {
      return SizedBox(
        height: widget.fillAvailable ? null : _stableBodyHeight,
        child: Center(child: Text('Failed to load users: $error')),
      );
    }

    if (response == null) {
      return _buildLoadingShell(theme);
    }

    final items = response.items;
    final total = response.pageInfo.total;
    final pageCount = total == 0 ? 1 : ((total + _pageSize - 1) ~/ _pageSize);
    final safePage = _page.clamp(1, pageCount);
    if (safePage != _page) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) setState(() => _page = safePage);
      });
    }

    final table = Card(
      clipBehavior: Clip.antiAlias,
      child: items.isEmpty
          ? SizedBox(
              height: widget.fillAvailable ? double.infinity : _emptyBodyHeight,
              child: const _EmptyTableMessage(
                icon: FluentIcons.people_24_regular,
                message: 'No users found',
              ),
            )
          : SizedBox(
              height: widget.fillAvailable ? null : _stableBodyHeight,
              child: Stack(
                children: [
                  Positioned.fill(
                    child: TableView.builder(
                      columns: _columns,
                      rowCount: items.length,
                      rowHeight: _rowHeight,
                      headerHeight: _headerHeight,
                      shrinkWrapVertical: false,
                      headerBuilder: (context, contentBuilder) {
                        return contentBuilder(
                          context,
                          (context, column) => _HeaderCell(
                            label: _headers[column],
                            style: theme.textTheme.titleSmall?.copyWith(
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        );
                      },
                      rowBuilder: (context, row, contentBuilder) {
                        final user = items[row];
                        return Material(
                          type: MaterialType.transparency,
                          child: contentBuilder(
                            context,
                            (context, column) => switch (column) {
                              0 => _CellText(user.nickname),
                              1 => _CellText(user.email ?? '-'),
                              2 => _CellText(user.phone ?? '-'),
                              _ => Align(
                                alignment: Alignment.center,
                                child: IconButton(
                                  icon: const Icon(
                                    FluentIcons.eye_24_regular,
                                    size: 20,
                                  ),
                                  tooltip: 'View',
                                  onPressed: () {
                                    // TODO: navigate to user detail
                                  },
                                ),
                              ),
                            },
                          ),
                        );
                      },
                    ),
                  ),
                  if (isLoading)
                    const Positioned(
                      top: 0,
                      left: 0,
                      right: 0,
                      child: LinearProgressIndicator(minHeight: 2),
                    ),
                ],
              ),
            ),
    );

    final pagination = _PaginationBar(
      page: safePage,
      pageSize: _pageSize,
      total: total,
      enabled: !isLoading,
      onPrevious: safePage > 1
          ? () => setState(() => _page = safePage - 1)
          : null,
      onNext: safePage < pageCount
          ? () => setState(() => _page = safePage + 1)
          : null,
    );

    if (widget.fillAvailable) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Expanded(child: table),
          pagination,
        ],
      );
    }

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [table, pagination],
    );
  }

  Widget _buildLoadingShell(ThemeData theme) {
    final table = Card(
      clipBehavior: Clip.antiAlias,
      child: SizedBox(
        height: widget.fillAvailable ? null : _stableBodyHeight,
        child: Stack(
          children: [
            Positioned.fill(
              child: TableView.builder(
                columns: _columns,
                rowCount: _pageSize,
                rowHeight: _rowHeight,
                headerHeight: _headerHeight,
                shrinkWrapVertical: false,
                headerBuilder: (context, contentBuilder) {
                  return contentBuilder(
                    context,
                    (context, column) => _HeaderCell(
                      label: _headers[column],
                      style: theme.textTheme.titleSmall?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  );
                },
                rowBuilder: (context, row, contentBuilder) => null,
              ),
            ),
            const Positioned(
              top: 0,
              left: 0,
              right: 0,
              child: LinearProgressIndicator(minHeight: 2),
            ),
          ],
        ),
      ),
    );

    final pagination = _PaginationBar(
      page: _page,
      pageSize: _pageSize,
      total: 0,
      enabled: false,
    );

    if (widget.fillAvailable) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Expanded(child: table),
          pagination,
        ],
      );
    }

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [table, pagination],
    );
  }
}

class _HeaderCell extends StatelessWidget {
  final String label;
  final TextStyle? style;

  const _HeaderCell({required this.label, this.style});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Align(
        alignment: Alignment.centerLeft,
        child: Text(label, style: style, overflow: TextOverflow.ellipsis),
      ),
    );
  }
}

class _CellText extends StatelessWidget {
  final String text;

  const _CellText(this.text);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Align(
        alignment: Alignment.centerLeft,
        child: Text(text, overflow: TextOverflow.ellipsis),
      ),
    );
  }
}

class _PaginationBar extends StatelessWidget {
  final int page;
  final int pageSize;
  final int total;
  final bool enabled;
  final VoidCallback? onPrevious;
  final VoidCallback? onNext;

  const _PaginationBar({
    required this.page,
    required this.pageSize,
    required this.total,
    this.enabled = true,
    this.onPrevious,
    this.onNext,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final start = total == 0 ? 0 : (page - 1) * pageSize + 1;
    final end = (page * pageSize).clamp(0, total);

    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          Text('$start–$end of $total', style: theme.textTheme.bodySmall),
          IconButton(
            icon: const Icon(FluentIcons.chevron_left_24_regular),
            tooltip: 'Previous page',
            onPressed: enabled ? onPrevious : null,
          ),
          IconButton(
            icon: const Icon(FluentIcons.chevron_right_24_regular),
            tooltip: 'Next page',
            onPressed: enabled ? onNext : null,
          ),
        ],
      ),
    );
  }
}

class _EmptyTableMessage extends StatelessWidget {
  final IconData icon;
  final String message;

  const _EmptyTableMessage({required this.icon, required this.message});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 40, color: theme.colorScheme.outline),
          const SizedBox(height: 12),
          Text(message, style: theme.textTheme.bodyMedium),
        ],
      ),
    );
  }
}
