import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_table_view/material_table_view.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../providers/audit_providers.dart';
import '../../widgets/admin_page_scaffold.dart';
import '../../widgets/table_cell_text.dart';
import 'audit_format.dart';
import 'audit_payload_dialog.dart';
import 'audit_table_widgets.dart';
import 'audit_type_filter.dart';

/// Platform-wide audit log list with type filter and payload dialog.
///
/// Backend: GET /audits (TenantRead)
class AuditsPage extends ConsumerStatefulWidget {
  const AuditsPage({super.key});

  @override
  ConsumerState<AuditsPage> createState() => _AuditsPageState();
}

class _AuditsPageState extends ConsumerState<AuditsPage> {
  static const _rowHeight = kMinInteractiveDimension;
  static const _headerHeight = 48.0;
  static const _pageSize = 25;

  static const _columns = [
    TableColumn(width: 120, flex: 2),
    TableColumn(width: 150, flex: 2),
    TableColumn(width: 200, flex: 4),
    TableColumn(width: 150, flex: 2),
    TableColumn(width: 48),
  ];

  static const _headers = ['ID', 'Type', 'Payload', 'Created', ''];

  int _page = 1;
  String? _auditType;
  PagedResponse<AuditLog>? _lastResponse;

  void _setAuditType(String? type) {
    if (type == _auditType) return;
    setState(() {
      _auditType = type;
      _page = 1;
      _lastResponse = null;
    });
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final async = ref.watch(auditsPageProvider(_page, _auditType));
    ref.listen(auditsPageProvider(_page, _auditType), (prev, next) {
      final data = next.valueOrNull;
      if (data != null && !identical(_lastResponse, data)) {
        setState(() => _lastResponse = data);
      }
    });

    return AdminPageScaffold(
      title: 'Audits',
      widthFactor: 0.9,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
            child: Align(
              alignment: Alignment.centerLeft,
              child: AuditTypeFilter(
                auditType: _auditType,
                onChanged: _setAuditType,
              ),
            ),
          ),
          Expanded(child: _buildBody(theme, async)),
        ],
      ),
    );
  }

  Widget _buildBody(
    ThemeData theme,
    AsyncValue<PagedResponse<AuditLog>> async,
  ) {
    final response = async.valueOrNull ?? _lastResponse;
    final isLoading = async.isLoading;
    final error = async.hasError && response == null ? async.error : null;

    if (error != null) {
      return Center(child: Text('Failed to load audits: $error'));
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
      margin: const EdgeInsets.symmetric(horizontal: 16),
      clipBehavior: Clip.antiAlias,
      child: items.isEmpty
          ? AuditEmptyMessage(
              icon: FluentIcons.document_search_24_regular,
              message: _auditType == null
                  ? 'No audit logs yet'
                  : 'No audits of type $_auditType',
            )
          : Stack(
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
                        (context, column) => AuditHeaderCell(
                          label: _headers[column],
                          style: theme.textTheme.titleSmall?.copyWith(
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                      );
                    },
                    rowBuilder: (context, row, contentBuilder) {
                      final log = items[row];
                      return Material(
                        type: MaterialType.transparency,
                        child: InkWell(
                          onTap: () => showAuditPayloadDialog(context, log),
                          child: contentBuilder(
                            context,
                            (context, column) => switch (column) {
                              0 => TableCellText(log.id),
                              1 => AuditTypeBadge(log.auditType),
                              2 => TableCellText(
                                auditPayloadPreview(log.payload),
                              ),
                              3 => TableCellText(
                                formatAuditCreatedAt(log.createdAt),
                              ),
                              _ => Align(
                                alignment: Alignment.center,
                                child: Icon(
                                  FluentIcons.eye_24_regular,
                                  size: 20,
                                  color: theme.colorScheme.onSurfaceVariant,
                                ),
                              ),
                            },
                          ),
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
    );

    final pagination = AuditPaginationBar(
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

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(child: table),
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
          child: pagination,
        ),
      ],
    );
  }

  Widget _buildLoadingShell(ThemeData theme) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(
          child: Card(
            margin: const EdgeInsets.symmetric(horizontal: 16),
            clipBehavior: Clip.antiAlias,
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
                        (context, column) => AuditHeaderCell(
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
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
          child: AuditPaginationBar(
            page: _page,
            pageSize: _pageSize,
            total: 0,
            enabled: false,
          ),
        ),
      ],
    );
  }
}
