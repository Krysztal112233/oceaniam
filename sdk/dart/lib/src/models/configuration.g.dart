// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'configuration.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$TokenConfigurationImpl _$$TokenConfigurationImplFromJson(
        Map<String, dynamic> json) =>
    _$TokenConfigurationImpl(
      issuer: json['issuer'] as String,
      audience:
          (json['audience'] as List<dynamic>).map((e) => e as String).toList(),
    );

Map<String, dynamic> _$$TokenConfigurationImplToJson(
        _$TokenConfigurationImpl instance) =>
    <String, dynamic>{
      'issuer': instance.issuer,
      'audience': instance.audience,
    };

_$Argon2ConfigurationImpl _$$Argon2ConfigurationImplFromJson(
        Map<String, dynamic> json) =>
    _$Argon2ConfigurationImpl(
      mCost: (json['m_cost'] as num).toInt(),
      tCost: (json['t_cost'] as num).toInt(),
      pCost: (json['p_cost'] as num).toInt(),
    );

Map<String, dynamic> _$$Argon2ConfigurationImplToJson(
        _$Argon2ConfigurationImpl instance) =>
    <String, dynamic>{
      'm_cost': instance.mCost,
      't_cost': instance.tCost,
      'p_cost': instance.pCost,
    };

_$PasswordConfigurationImpl _$$PasswordConfigurationImplFromJson(
        Map<String, dynamic> json) =>
    _$PasswordConfigurationImpl(
      argon2:
          Argon2Configuration.fromJson(json['argon2'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$$PasswordConfigurationImplToJson(
        _$PasswordConfigurationImpl instance) =>
    <String, dynamic>{
      'argon2': instance.argon2,
    };

_$AuthConfigurationImpl _$$AuthConfigurationImplFromJson(
        Map<String, dynamic> json) =>
    _$AuthConfigurationImpl(
      token: TokenConfiguration.fromJson(json['token'] as Map<String, dynamic>),
      password: PasswordConfiguration.fromJson(
          json['password'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$$AuthConfigurationImplToJson(
        _$AuthConfigurationImpl instance) =>
    <String, dynamic>{
      'token': instance.token,
      'password': instance.password,
    };

_$RegistrationConfigurationImpl _$$RegistrationConfigurationImplFromJson(
        Map<String, dynamic> json) =>
    _$RegistrationConfigurationImpl(
      enabled: json['enabled'] as bool,
    );

Map<String, dynamic> _$$RegistrationConfigurationImplToJson(
        _$RegistrationConfigurationImpl instance) =>
    <String, dynamic>{
      'enabled': instance.enabled,
    };

_$ApplicationConfigurationImpl _$$ApplicationConfigurationImplFromJson(
        Map<String, dynamic> json) =>
    _$ApplicationConfigurationImpl(
      auth: AuthConfiguration.fromJson(json['auth'] as Map<String, dynamic>),
      registration: RegistrationConfiguration.fromJson(
          json['registration'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$$ApplicationConfigurationImplToJson(
        _$ApplicationConfigurationImpl instance) =>
    <String, dynamic>{
      'auth': instance.auth,
      'registration': instance.registration,
    };

_$PatchTokenConfigurationImpl _$$PatchTokenConfigurationImplFromJson(
        Map<String, dynamic> json) =>
    _$PatchTokenConfigurationImpl(
      issuer: json['issuer'] as String?,
      audience: (json['audience'] as List<dynamic>?)
          ?.map((e) => e as String)
          .toList(),
    );

Map<String, dynamic> _$$PatchTokenConfigurationImplToJson(
        _$PatchTokenConfigurationImpl instance) =>
    <String, dynamic>{
      if (instance.issuer case final value?) 'issuer': value,
      if (instance.audience case final value?) 'audience': value,
    };

_$PatchAuthConfigurationImpl _$$PatchAuthConfigurationImplFromJson(
        Map<String, dynamic> json) =>
    _$PatchAuthConfigurationImpl(
      token: json['token'] == null
          ? null
          : PatchTokenConfiguration.fromJson(
              json['token'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$$PatchAuthConfigurationImplToJson(
        _$PatchAuthConfigurationImpl instance) =>
    <String, dynamic>{
      if (instance.token?.toJson() case final value?) 'token': value,
    };

_$PatchRegistrationConfigurationImpl
    _$$PatchRegistrationConfigurationImplFromJson(Map<String, dynamic> json) =>
        _$PatchRegistrationConfigurationImpl(
          enabled: json['enabled'] as bool?,
        );

Map<String, dynamic> _$$PatchRegistrationConfigurationImplToJson(
        _$PatchRegistrationConfigurationImpl instance) =>
    <String, dynamic>{
      if (instance.enabled case final value?) 'enabled': value,
    };

_$PatchApplicationConfigurationImpl
    _$$PatchApplicationConfigurationImplFromJson(Map<String, dynamic> json) =>
        _$PatchApplicationConfigurationImpl(
          auth: json['auth'] == null
              ? null
              : PatchAuthConfiguration.fromJson(
                  json['auth'] as Map<String, dynamic>),
          registration: json['registration'] == null
              ? null
              : PatchRegistrationConfiguration.fromJson(
                  json['registration'] as Map<String, dynamic>),
        );

Map<String, dynamic> _$$PatchApplicationConfigurationImplToJson(
        _$PatchApplicationConfigurationImpl instance) =>
    <String, dynamic>{
      if (instance.auth?.toJson() case final value?) 'auth': value,
      if (instance.registration?.toJson() case final value?)
        'registration': value,
    };
