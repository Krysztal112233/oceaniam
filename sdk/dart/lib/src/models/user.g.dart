// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'user.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ApplicationUserImpl _$$ApplicationUserImplFromJson(
        Map<String, dynamic> json) =>
    _$ApplicationUserImpl(
      id: json['id'] as String,
      email: json['email'] as String?,
      phone: json['phone'] as String?,
      nickname: json['nickname'] as String,
    );

Map<String, dynamic> _$$ApplicationUserImplToJson(
        _$ApplicationUserImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'email': instance.email,
      'phone': instance.phone,
      'nickname': instance.nickname,
    };

_$CreateUserRequestImpl _$$CreateUserRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$CreateUserRequestImpl(
      nickname: json['nickname'] as String,
      password: json['password'] as String,
      email: json['email'] as String?,
      phone: json['phone'] as String?,
    );

Map<String, dynamic> _$$CreateUserRequestImplToJson(
        _$CreateUserRequestImpl instance) =>
    <String, dynamic>{
      'nickname': instance.nickname,
      'password': instance.password,
      'email': instance.email,
      'phone': instance.phone,
    };

_$UpdatePasswordRequestImpl _$$UpdatePasswordRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$UpdatePasswordRequestImpl(
      password: json['password'] as String,
    );

Map<String, dynamic> _$$UpdatePasswordRequestImplToJson(
        _$UpdatePasswordRequestImpl instance) =>
    <String, dynamic>{
      'password': instance.password,
    };

_$UserSearchQueryImpl _$$UserSearchQueryImplFromJson(
        Map<String, dynamic> json) =>
    _$UserSearchQueryImpl(
      query: json['query'] as String?,
      searchBy: json['search_by'] as String?,
    );

Map<String, dynamic> _$$UserSearchQueryImplToJson(
        _$UserSearchQueryImpl instance) =>
    <String, dynamic>{
      'query': instance.query,
      'search_by': instance.searchBy,
    };
