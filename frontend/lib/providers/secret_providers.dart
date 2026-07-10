import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import 'oceaniam_client_provider.dart';

part 'secret_providers.g.dart';

@riverpod
Future<PagedResponse<Secret>> secretsPage(Ref ref, int page) async {
  final client = ref.watch(oceanIAMClientProvider);
  return client.listSecrets(page: page, pageSize: 20);
}
