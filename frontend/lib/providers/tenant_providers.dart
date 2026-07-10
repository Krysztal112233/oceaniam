import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import 'oceaniam_client_provider.dart';
import 'shared_preferences_provider.dart';

part 'tenant_providers.g.dart';

const _kStoredTenantIdKey = 'current_tenant_id';

@Riverpod(keepAlive: true)
class CurrentTenantId extends _$CurrentTenantId {
  @override
  String? build() {
    final prefs = ref.watch(sharedPreferencesProvider);
    return prefs.getString(_kStoredTenantIdKey);
  }

  void select(String? tenantId) {
    state = tenantId;
    final prefs = ref.read(sharedPreferencesProvider);
    if (tenantId != null) {
      prefs.setString(_kStoredTenantIdKey, tenantId);
    } else {
      prefs.remove(_kStoredTenantIdKey);
    }
  }
}

@Riverpod(keepAlive: true)
Future<List<Tenant>> tenantList(Ref ref) async {
  final client = ref.watch(oceanIAMClientProvider);
  final first = await client.listTenants(page: 1, pageSize: 100);
  return first.items;
}

@Riverpod(keepAlive: true)
Tenant? currentTenant(Ref ref) {
  final id = ref.watch(currentTenantIdProvider);
  if (id == null) return null;
  final tenants = ref.watch(tenantListProvider);
  return tenants.valueOrNull?.where((t) => t.id == id).firstOrNull;
}
