// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'tenant.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$TenantImpl _$$TenantImplFromJson(Map<String, dynamic> json) => _$TenantImpl(
      id: json['id'] as String,
      comment: json['comment'] as String?,
    );

Map<String, dynamic> _$$TenantImplToJson(_$TenantImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'comment': instance.comment,
    };

_$CreateTenantRequestImpl _$$CreateTenantRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$CreateTenantRequestImpl(
      comment: json['comment'] as String?,
    );

Map<String, dynamic> _$$CreateTenantRequestImplToJson(
        _$CreateTenantRequestImpl instance) =>
    <String, dynamic>{
      'comment': instance.comment,
    };

_$UpdateTenantRequestImpl _$$UpdateTenantRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$UpdateTenantRequestImpl(
      comment: json['comment'] as String?,
    );

Map<String, dynamic> _$$UpdateTenantRequestImplToJson(
        _$UpdateTenantRequestImpl instance) =>
    <String, dynamic>{
      'comment': instance.comment,
    };
