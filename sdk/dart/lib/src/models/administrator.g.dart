// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'administrator.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$AdministratorImpl _$$AdministratorImplFromJson(Map<String, dynamic> json) =>
    _$AdministratorImpl(
      id: json['id'] as String,
      name: json['name'] as String,
      role: json['role'] as String?,
    );

Map<String, dynamic> _$$AdministratorImplToJson(_$AdministratorImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'role': instance.role,
    };

_$AdministratorProfileImpl _$$AdministratorProfileImplFromJson(
        Map<String, dynamic> json) =>
    _$AdministratorProfileImpl(
      id: json['id'] as String,
      name: json['name'] as String,
      role: json['role'] as String?,
      permissions: (json['permissions'] as List<dynamic>)
          .map((e) => e as String)
          .toList(),
    );

Map<String, dynamic> _$$AdministratorProfileImplToJson(
        _$AdministratorProfileImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'role': instance.role,
      'permissions': instance.permissions,
    };

_$CreateAdministratorRequestImpl _$$CreateAdministratorRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$CreateAdministratorRequestImpl(
      name: json['name'] as String,
      password: json['password'] as String,
    );

Map<String, dynamic> _$$CreateAdministratorRequestImplToJson(
        _$CreateAdministratorRequestImpl instance) =>
    <String, dynamic>{
      'name': instance.name,
      'password': instance.password,
    };

_$CreateAdministratorResponseImpl _$$CreateAdministratorResponseImplFromJson(
        Map<String, dynamic> json) =>
    _$CreateAdministratorResponseImpl(
      id: json['id'] as String,
      name: json['name'] as String,
      password: json['password'] as String,
    );

Map<String, dynamic> _$$CreateAdministratorResponseImplToJson(
        _$CreateAdministratorResponseImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'password': instance.password,
    };

_$UpdateAdministratorRequestImpl _$$UpdateAdministratorRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$UpdateAdministratorRequestImpl(
      name: json['name'] as String?,
      password: json['password'] as String?,
    );

Map<String, dynamic> _$$UpdateAdministratorRequestImplToJson(
        _$UpdateAdministratorRequestImpl instance) =>
    <String, dynamic>{
      'name': instance.name,
      'password': instance.password,
    };
