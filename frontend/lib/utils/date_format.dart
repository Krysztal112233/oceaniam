/// Formats an ISO-8601 timestamp as `yyyy-MM-dd HH:mm` in local time.
/// Returns the raw string when it cannot be parsed.
String formatDateTimeMinute(String raw) {
  final dt = DateTime.tryParse(raw);
  if (dt == null) return raw;
  final local = dt.toLocal();
  String two(int n) => n.toString().padLeft(2, '0');
  return '${local.year}-${two(local.month)}-${two(local.day)} '
      '${two(local.hour)}:${two(local.minute)}';
}
