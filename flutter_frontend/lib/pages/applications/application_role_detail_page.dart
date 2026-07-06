import 'package:flutter/material.dart';

import '../../widgets/placeholder_page.dart';

/// 角色详情：权限矩阵 + 主体分配/取消。
///
/// 后端：
/// - GET /tenants/{tid}/applications/{aid}/roles/{rid}
/// - PATCH /tenants/{tid}/applications/{aid}/roles/{rid}
/// - DELETE /tenants/{tid}/applications/{aid}/roles/{rid}
/// - GET /tenants/{tid}/applications/{aid}/roles/{rid}/permissions
/// - PUT /tenants/{tid}/applications/{aid}/roles/{rid}/permissions
/// - GET /tenants/{tid}/applications/{aid}/roles/subjects/{sid}/roles
/// - POST /tenants/{tid}/applications/{aid}/roles/subjects/{sid}/roles
/// - DELETE /tenants/{tid}/applications/{aid}/roles/subjects/{sid}/roles/{rid}
class ApplicationRoleDetailPage extends StatelessWidget {
  final String tenantId;
  final String applicationId;
  final String roleId;

  const ApplicationRoleDetailPage({
    super.key,
    required this.tenantId,
    required this.applicationId,
    required this.roleId,
  });

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Role $roleId',
      description: 'Permissions matrix and subject assignments.',
    );
  }
}
