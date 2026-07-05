import 'package:freezed_annotation/freezed_annotation.dart';

part 'audit.freezed.dart';
part 'audit.g.dart';

@freezed
class AuditLog with _$AuditLog {
  const factory AuditLog({
    required String id,
    @JsonKey(name: 'audit_type') required String auditType,
    required Map<String, dynamic> payload,
    @JsonKey(name: 'created_at') required String createdAt,
  }) = _AuditLog;

  factory AuditLog.fromJson(Map<String, dynamic> json) =>
      _$AuditLogFromJson(json);
}
