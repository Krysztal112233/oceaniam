import 'dart:convert';

import '../../utils/date_format.dart';

String auditPayloadPreview(Map<String, dynamic> payload, {int maxLen = 80}) {
  final encoded = jsonEncode(payload);
  if (encoded.length <= maxLen) return encoded;
  return '${encoded.substring(0, maxLen - 3)}…';
}

String formatAuditCreatedAt(String raw) => formatDateTimeMinute(raw);
