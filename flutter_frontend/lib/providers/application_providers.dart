import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import 'oceaniam_client_provider.dart';
import 'tenant_providers.dart';

part 'application_providers.g.dart';

@Riverpod(keepAlive: true)
Future<List<Application>> applicationList(Ref ref) async {
  final tenantId = ref.watch(currentTenantIdProvider);
  if (tenantId == null) return [];
  final client = ref.watch(oceanIAMClientProvider);
  final first = await client.listApplications(tenantId, page: 1, pageSize: 100);
  return first.items;
}

@riverpod
Future<List<ApplicationUser>> applicationUsers(
  Ref ref,
  String tenantId,
  String applicationId,
) async {
  final client = ref.watch(oceanIAMClientProvider);
  final response = await client.listUsers(
    tenantId,
    applicationId,
    page: 1,
    pageSize: 100,
  );
  return response.items;
}
