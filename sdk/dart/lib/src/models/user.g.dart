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

_$SearchApplicationUsersQueryImpl _$$SearchApplicationUsersQueryImplFromJson(
        Map<String, dynamic> json) =>
    _$SearchApplicationUsersQueryImpl(
      page: (json['page'] as num?)?.toInt() ?? 1,
      perPage: (json['per_page'] as num?)?.toInt() ?? 30,
      sortOrder: json['sort_order'] as String?,
      byNickname: json['by_nickname'] as String?,
      byEmail: json['by_email'] as String?,
      byPhone: json['by_phone'] as String?,
      byId: json['by_id'] as String?,
    );

Map<String, dynamic> _$$SearchApplicationUsersQueryImplToJson(
        _$SearchApplicationUsersQueryImpl instance) =>
    <String, dynamic>{
      'page': instance.page,
      'per_page': instance.perPage,
      'sort_order': instance.sortOrder,
      'by_nickname': instance.byNickname,
      'by_email': instance.byEmail,
      'by_phone': instance.byPhone,
      'by_id': instance.byId,
    };
