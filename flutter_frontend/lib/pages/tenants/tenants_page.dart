import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../widgets/placeholder_page.dart';

/// 租户分页列表。
///
/// 后端：GET /tenants（TenantRead），POST /tenants（TenantCreate）。
class TenantsPage extends StatelessWidget {
  const TenantsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Tenants',
      description:
          'Paginated list of tenants. Create, search, drill into detail.',
      actions: [
        FilledButton.icon(
          key: const Key('create-tenant'),
          onPressed: () {
            // TODO: POST /tenants
          },
          icon: const Icon(FluentIcons.add_24_regular),
          label: const Text('New tenant'),
        ),
      ],
    );
  }
}
