// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'configuration.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$AuthConfigImpl _$$AuthConfigImplFromJson(Map<String, dynamic> json) =>
    _$AuthConfigImpl(
      tokenIssuer: json['token_issuer'] as String?,
      tokenAudience: json['token_audience'] as String?,
      passwordPolicy: json['password_policy'] == null
          ? null
          : PasswordPolicy.fromJson(
              json['password_policy'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$$AuthConfigImplToJson(_$AuthConfigImpl instance) =>
    <String, dynamic>{
      'token_issuer': instance.tokenIssuer,
      'token_audience': instance.tokenAudience,
      'password_policy': instance.passwordPolicy,
    };

_$PasswordPolicyImpl _$$PasswordPolicyImplFromJson(Map<String, dynamic> json) =>
    _$PasswordPolicyImpl(
      minLength: (json['min_length'] as num?)?.toInt(),
      requireUppercase: json['require_uppercase'] as bool?,
      requireLowercase: json['require_lowercase'] as bool?,
      requireDigit: json['require_digit'] as bool?,
      requireSpecial: json['require_special'] as bool?,
    );

Map<String, dynamic> _$$PasswordPolicyImplToJson(
        _$PasswordPolicyImpl instance) =>
    <String, dynamic>{
      'min_length': instance.minLength,
      'require_uppercase': instance.requireUppercase,
      'require_lowercase': instance.requireLowercase,
      'require_digit': instance.requireDigit,
      'require_special': instance.requireSpecial,
    };

_$RegistrationConfigImpl _$$RegistrationConfigImplFromJson(
        Map<String, dynamic> json) =>
    _$RegistrationConfigImpl(
      enabled: json['enabled'] as bool?,
    );

Map<String, dynamic> _$$RegistrationConfigImplToJson(
        _$RegistrationConfigImpl instance) =>
    <String, dynamic>{
      'enabled': instance.enabled,
    };

_$ApplicationConfigurationImpl _$$ApplicationConfigurationImplFromJson(
        Map<String, dynamic> json) =>
    _$ApplicationConfigurationImpl(
      auth: json['auth'] == null
          ? null
          : AuthConfig.fromJson(json['auth'] as Map<String, dynamic>),
      registration: json['registration'] == null
          ? null
          : RegistrationConfig.fromJson(
              json['registration'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$$ApplicationConfigurationImplToJson(
        _$ApplicationConfigurationImpl instance) =>
    <String, dynamic>{
      'auth': instance.auth,
      'registration': instance.registration,
    };
