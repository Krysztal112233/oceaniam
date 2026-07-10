import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import 'oceaniam_client_provider.dart';

part 'audit_providers.g.dart';

/// Values match backend `AuditType` strum Display / API `audit_type` strings.
const kAuditTypes = <String>[
  'SignJwt',
  'RevokeJwt',
  'RefreshJwt',
  'CreateApplication',
  'PatchApplication',
  'PatchApplicationConfiguration',
  'DeleteApplication',
  'CreateTenants',
  'DeleteTenants',
  'PatchTenant',
  'CreateAdministrator',
  'PatchAdministrator',
  'CreateApplicationUser',
  'DeleteApplicationUser',
  'CreateApplicationSecret',
  'DeleteApplicationSecret',
  'BindApplicationSecret',
  'UnbindApplicationSecret',
  'CreateChallenge',
  'VerifyChallenge',
  'RotateKey',
  'RevokeKey',
];

@riverpod
Future<PagedResponse<AuditLog>> auditsPage(
  Ref ref,
  int page,
  String? auditType,
) async {
  final client = ref.watch(oceanIAMClientProvider);
  return client.listAudits(page: page, pageSize: 25, auditType: auditType);
}
