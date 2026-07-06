// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'auth.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$SigninRequestImpl _$$SigninRequestImplFromJson(Map<String, dynamic> json) =>
    _$SigninRequestImpl(
      name: json['name'] as String,
      password: json['password'] as String,
    );

Map<String, dynamic> _$$SigninRequestImplToJson(_$SigninRequestImpl instance) =>
    <String, dynamic>{
      'name': instance.name,
      'password': instance.password,
    };

_$SigninResponseImpl _$$SigninResponseImplFromJson(Map<String, dynamic> json) =>
    _$SigninResponseImpl(
      jwt: json['jwt'] as String,
    );

Map<String, dynamic> _$$SigninResponseImplToJson(
        _$SigninResponseImpl instance) =>
    <String, dynamic>{
      'jwt': instance.jwt,
    };

_$RefreshTokenResponseImpl _$$RefreshTokenResponseImplFromJson(
        Map<String, dynamic> json) =>
    _$RefreshTokenResponseImpl(
      jwt: json['jwt'] as String,
    );

Map<String, dynamic> _$$RefreshTokenResponseImplToJson(
        _$RefreshTokenResponseImpl instance) =>
    <String, dynamic>{
      'jwt': instance.jwt,
    };
