import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import 'oceaniam_client_provider.dart';
import 'tenant_providers.dart';

part 'application_providers.g.dart';

/// Field used for application user search (`GET .../users/search`).
enum ApplicationUserSearchField { nickname, email, phone, id }

@Riverpod(keepAlive: true)
Future<List<Application>> applicationList(Ref ref) async {
  final tenantId = ref.watch(currentTenantIdProvider);
  if (tenantId == null) return [];
  final client = ref.watch(oceanIAMClientProvider);
  final first = await client.listApplications(tenantId, page: 1, pageSize: 100);
  return first.items;
}

/// Application metadata for the Overview tab.
@riverpod
Future<ApplicationDetail> applicationDetail(
  Ref ref,
  String tenantId,
  String applicationId,
) async {
  final client = ref.watch(oceanIAMClientProvider);
  return client.getApplication(tenantId, applicationId);
}

/// Lists users, or searches when [searchQuery] is non-empty.
///
/// [searchField] selects which `by_*` query param is sent. Empty [searchQuery]
/// falls back to the paginated list endpoint.
@riverpod
Future<PagedResponse<ApplicationUser>> applicationUsersPage(
  Ref ref,
  String tenantId,
  String applicationId,
  int page,
  ApplicationUserSearchField searchField,
  String searchQuery,
) async {
  final client = ref.watch(oceanIAMClientProvider);
  final query = searchQuery.trim();
  if (query.isEmpty) {
    return client.listUsers(tenantId, applicationId, page: page, pageSize: 10);
  }

  final sanitized = query
      .replaceAll('%', '')
      .replaceAll('_', '')
      .replaceAll(r'\', '');
  if (sanitized.isEmpty) {
    return client.listUsers(tenantId, applicationId, page: page, pageSize: 10);
  }

  return client.searchUsers(
    tenantId,
    applicationId,
    SearchApplicationUsersQuery(
      page: page,
      perPage: 10,
      byNickname: searchField == ApplicationUserSearchField.nickname
          ? sanitized
          : null,
      byEmail: searchField == ApplicationUserSearchField.email
          ? sanitized
          : null,
      byPhone: searchField == ApplicationUserSearchField.phone
          ? sanitized
          : null,
      byId: searchField == ApplicationUserSearchField.id ? sanitized : null,
    ),
  );
}
