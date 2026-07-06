import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../../widgets/placeholder_page.dart';

/// 应用用户详情：凭据管理（密码重置 / TOTP enroll/verify/remove）。
///
/// 后端：
/// - GET /tenants/{tid}/applications/{aid}/users/{uid}
/// - PATCH /tenants/{tid}/applications/{aid}/users/{uid}/credentials
/// - POST /tenants/{tid}/applications/{aid}/users/{uid}/totp/enroll
/// - POST /tenants/{tid}/applications/{aid}/users/{uid}/totp/verify
/// - DELETE /tenants/{tid}/applications/{aid}/users/{uid}/totp
class ApplicationUserDetailPage extends StatelessWidget {
  final String tenantId;
  final String applicationId;
  final String userId;

  const ApplicationUserDetailPage({
    super.key,
    required this.tenantId,
    required this.applicationId,
    required this.userId,
  });

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'User $userId',
      description: 'Profile, password reset, TOTP enrollment.',
    );
  }
}
