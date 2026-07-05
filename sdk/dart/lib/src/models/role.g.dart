// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'role.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ApplicationRoleImpl _$$ApplicationRoleImplFromJson(
        Map<String, dynamic> json) =>
    _$ApplicationRoleImpl(
      id: json['id'] as String,
      name: json['name'] as String,
      isSystem: json['is_system'] as bool,
      permissions: (json['permissions'] as List<dynamic>)
          .map((e) => e as String)
          .toList(),
    );

Map<String, dynamic> _$$ApplicationRoleImplToJson(
        _$ApplicationRoleImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'is_system': instance.isSystem,
      'permissions': instance.permissions,
    };

_$SubjectRolesImpl _$$SubjectRolesImplFromJson(Map<String, dynamic> json) =>
    _$SubjectRolesImpl(
      subjectId: json['subject_id'] as String,
      roleIds:
          (json['role_ids'] as List<dynamic>).map((e) => e as String).toList(),
    );

Map<String, dynamic> _$$SubjectRolesImplToJson(_$SubjectRolesImpl instance) =>
    <String, dynamic>{
      'subject_id': instance.subjectId,
      'role_ids': instance.roleIds,
    };

_$RolePermissionsImpl _$$RolePermissionsImplFromJson(
        Map<String, dynamic> json) =>
    _$RolePermissionsImpl(
      permissions: (json['permissions'] as List<dynamic>)
          .map((e) => e as String)
          .toList(),
    );

Map<String, dynamic> _$$RolePermissionsImplToJson(
        _$RolePermissionsImpl instance) =>
    <String, dynamic>{
      'permissions': instance.permissions,
    };

_$CreateRoleRequestImpl _$$CreateRoleRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$CreateRoleRequestImpl(
      name: json['name'] as String,
      permissions: (json['permissions'] as List<dynamic>)
          .map((e) => e as String)
          .toList(),
    );

Map<String, dynamic> _$$CreateRoleRequestImplToJson(
        _$CreateRoleRequestImpl instance) =>
    <String, dynamic>{
      'name': instance.name,
      'permissions': instance.permissions,
    };

_$UpdateRoleRequestImpl _$$UpdateRoleRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$UpdateRoleRequestImpl(
      name: json['name'] as String?,
    );

Map<String, dynamic> _$$UpdateRoleRequestImplToJson(
        _$UpdateRoleRequestImpl instance) =>
    <String, dynamic>{
      'name': instance.name,
    };

_$UpdateRolePermissionsRequestImpl _$$UpdateRolePermissionsRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$UpdateRolePermissionsRequestImpl(
      permissions: (json['permissions'] as List<dynamic>)
          .map((e) => e as String)
          .toList(),
    );

Map<String, dynamic> _$$UpdateRolePermissionsRequestImplToJson(
        _$UpdateRolePermissionsRequestImpl instance) =>
    <String, dynamic>{
      'permissions': instance.permissions,
    };
