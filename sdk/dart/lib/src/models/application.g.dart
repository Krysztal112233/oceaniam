// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'application.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ApplicationImpl _$$ApplicationImplFromJson(Map<String, dynamic> json) =>
    _$ApplicationImpl(
      id: json['id'] as String,
      comment: json['comment'] as String?,
      tenantId: json['tenant_id'] as String,
    );

Map<String, dynamic> _$$ApplicationImplToJson(_$ApplicationImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'comment': instance.comment,
      'tenant_id': instance.tenantId,
    };

_$ApplicationDetailImpl _$$ApplicationDetailImplFromJson(
        Map<String, dynamic> json) =>
    _$ApplicationDetailImpl(
      id: json['id'] as String,
      comment: json['comment'] as String?,
      tenantId: json['tenant_id'] as String,
    );

Map<String, dynamic> _$$ApplicationDetailImplToJson(
        _$ApplicationDetailImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'comment': instance.comment,
      'tenant_id': instance.tenantId,
    };

_$CreateApplicationRequestImpl _$$CreateApplicationRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$CreateApplicationRequestImpl(
      comment: json['comment'] as String?,
    );

Map<String, dynamic> _$$CreateApplicationRequestImplToJson(
        _$CreateApplicationRequestImpl instance) =>
    <String, dynamic>{
      'comment': instance.comment,
    };

_$UpdateApplicationRequestImpl _$$UpdateApplicationRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$UpdateApplicationRequestImpl(
      comment: json['comment'] as String?,
    );

Map<String, dynamic> _$$UpdateApplicationRequestImplToJson(
        _$UpdateApplicationRequestImpl instance) =>
    <String, dynamic>{
      'comment': instance.comment,
    };

_$CreateApplicationResponseImpl _$$CreateApplicationResponseImplFromJson(
        Map<String, dynamic> json) =>
    _$CreateApplicationResponseImpl(
      applicationId: json['application_id'] as String,
      tenantId: json['tenant_id'] as String,
      comment: json['comment'] as String?,
    );

Map<String, dynamic> _$$CreateApplicationResponseImplToJson(
        _$CreateApplicationResponseImpl instance) =>
    <String, dynamic>{
      'application_id': instance.applicationId,
      'tenant_id': instance.tenantId,
      'comment': instance.comment,
    };
