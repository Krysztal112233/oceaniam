// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'secret.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$SecretImpl _$$SecretImplFromJson(Map<String, dynamic> json) => _$SecretImpl(
      id: json['id'] as String,
      secret: json['secret'] as String?,
      createdAt: json['created_at'] as String,
      revokedAt: json['revoked_at'] as String?,
      applicationIds: (json['application_ids'] as List<dynamic>)
          .map((e) => e as String)
          .toList(),
    );

Map<String, dynamic> _$$SecretImplToJson(_$SecretImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'secret': instance.secret,
      'created_at': instance.createdAt,
      'revoked_at': instance.revokedAt,
      'application_ids': instance.applicationIds,
    };

_$CreateSecretResponseImpl _$$CreateSecretResponseImplFromJson(
        Map<String, dynamic> json) =>
    _$CreateSecretResponseImpl(
      id: json['id'] as String,
      secret: json['secret'] as String,
    );

Map<String, dynamic> _$$CreateSecretResponseImplToJson(
        _$CreateSecretResponseImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'secret': instance.secret,
    };

_$BindSecretRequestImpl _$$BindSecretRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$BindSecretRequestImpl(
      applicationId: json['application_id'] as String,
    );

Map<String, dynamic> _$$BindSecretRequestImplToJson(
        _$BindSecretRequestImpl instance) =>
    <String, dynamic>{
      'application_id': instance.applicationId,
    };
