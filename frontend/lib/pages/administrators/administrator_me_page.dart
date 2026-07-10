import 'package:flutter/material.dart';

import '../../widgets/placeholder_page.dart';

/// 当前管理员个人资料 + 已解析权限集。
///
/// 后端：
/// - GET /administrators/me
/// - PATCH /administrators/{aid}（AdministratorPatch，自改 name/password）
class AdministratorMePage extends StatelessWidget {
  const AdministratorMePage({super.key});

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'My profile',
      description: 'Profile, resolved permission set, change name/password.',
    );
  }
}
