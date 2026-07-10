import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'oceaniam_client_provider.dart';

part 'key_providers.g.dart';

@riverpod
Future<List<ApplicationKey>> tenantKeys(Ref ref, String tenantId) async {
  final client = ref.watch(oceanIAMClientProvider);
  return client.listKeys(tenantId);
}
