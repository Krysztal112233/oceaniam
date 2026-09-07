// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'oidc_client.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$OidcClientImpl _$$OidcClientImplFromJson(Map<String, dynamic> json) =>
    _$OidcClientImpl(
      clientId: json['client_id'] as String,
      applicationId: json['application_id'] as String,
      name: json['name'] as String,
      clientType: json['client_type'] as String,
      applicationType: json['application_type'] as String,
      redirectUris: (json['redirect_uris'] as List<dynamic>)
          .map((e) => e as String)
          .toList(),
      createdAt: json['created_at'] as String,
    );

Map<String, dynamic> _$$OidcClientImplToJson(_$OidcClientImpl instance) =>
    <String, dynamic>{
      'client_id': instance.clientId,
      'application_id': instance.applicationId,
      'name': instance.name,
      'client_type': instance.clientType,
      'application_type': instance.applicationType,
      'redirect_uris': instance.redirectUris,
      'created_at': instance.createdAt,
    };

_$CreateOidcClientRequestImpl _$$CreateOidcClientRequestImplFromJson(
        Map<String, dynamic> json) =>
    _$CreateOidcClientRequestImpl(
      name: json['name'] as String,
      redirectUris: (json['redirect_uris'] as List<dynamic>)
          .map((e) => e as String)
          .toList(),
    );

Map<String, dynamic> _$$CreateOidcClientRequestImplToJson(
        _$CreateOidcClientRequestImpl instance) =>
    <String, dynamic>{
      'name': instance.name,
      'redirect_uris': instance.redirectUris,
    };
