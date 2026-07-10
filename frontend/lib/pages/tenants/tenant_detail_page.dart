import 'package:flutter/material.dart';

import '../../widgets/placeholder_page.dart';

/// 租户详情（含 tabs：概览 / 用户 / 密钥 / JWKS）。
///
/// 后端：
/// - GET /tenants/{tid}（TenantRead）
/// - PATCH /tenants/{tid}（TenantPatch）
/// - DELETE /tenants/{tid}（TenantDelete）
/// - GET /tenants/{tid}/users（TenantRead）
/// - GET /tenants/{tid}/.well-known/jwks.json（公开）
class TenantDetailPage extends StatelessWidget {
  final String tenantId;

  const TenantDetailPage({super.key, required this.tenantId});

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Tenant $tenantId',
      description: 'Overview, users, keys, JWKS.',
    );
  }
}
