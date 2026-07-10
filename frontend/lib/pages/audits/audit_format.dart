import 'dart:convert';

String auditPayloadPreview(Map<String, dynamic> payload, {int maxLen = 80}) {
  final encoded = jsonEncode(payload);
  if (encoded.length <= maxLen) return encoded;
  return '${encoded.substring(0, maxLen - 3)}…';
}

String formatAuditCreatedAt(String raw) {
  final dt = DateTime.tryParse(raw);
  if (dt == null) return raw;
  final local = dt.toLocal();
  String two(int n) => n.toString().padLeft(2, '0');
  return '${local.year}-${two(local.month)}-${two(local.day)} '
      '${two(local.hour)}:${two(local.minute)}:${two(local.second)}';
}
