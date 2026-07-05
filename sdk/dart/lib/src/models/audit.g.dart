// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'audit.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$AuditLogImpl _$$AuditLogImplFromJson(Map<String, dynamic> json) =>
    _$AuditLogImpl(
      id: json['id'] as String,
      auditType: json['audit_type'] as String,
      payload: json['payload'] as Map<String, dynamic>,
      createdAt: json['created_at'] as String,
    );

Map<String, dynamic> _$$AuditLogImplToJson(_$AuditLogImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'audit_type': instance.auditType,
      'payload': instance.payload,
      'created_at': instance.createdAt,
    };
