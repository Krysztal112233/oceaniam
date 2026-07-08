import 'package:freezed_annotation/freezed_annotation.dart';

part 'configuration.freezed.dart';
part 'configuration.g.dart';

@freezed
class AuthConfig with _$AuthConfig {
  const factory AuthConfig({
    @JsonKey(name: 'token_issuer') String? tokenIssuer,
    @JsonKey(name: 'token_audience') String? tokenAudience,
    @JsonKey(name: 'password_policy') PasswordPolicy? passwordPolicy,
  }) = _AuthConfig;

  factory AuthConfig.fromJson(Map<String, dynamic> json) =>
      _$AuthConfigFromJson(json);
}

@freezed
class PasswordPolicy with _$PasswordPolicy {
  const factory PasswordPolicy({
    @JsonKey(name: 'min_length') int? minLength,
    @JsonKey(name: 'require_uppercase') bool? requireUppercase,
    @JsonKey(name: 'require_lowercase') bool? requireLowercase,
    @JsonKey(name: 'require_digit') bool? requireDigit,
    @JsonKey(name: 'require_special') bool? requireSpecial,
  }) = _PasswordPolicy;

  factory PasswordPolicy.fromJson(Map<String, dynamic> json) =>
      _$PasswordPolicyFromJson(json);
}

@freezed
class RegistrationConfig with _$RegistrationConfig {
  const factory RegistrationConfig({bool? enabled}) = _RegistrationConfig;

  factory RegistrationConfig.fromJson(Map<String, dynamic> json) =>
      _$RegistrationConfigFromJson(json);
}

@freezed
class ApplicationConfiguration with _$ApplicationConfiguration {
  const factory ApplicationConfiguration({
    AuthConfig? auth,
    RegistrationConfig? registration,
  }) = _ApplicationConfiguration;

  factory ApplicationConfiguration.fromJson(Map<String, dynamic> json) =>
      _$ApplicationConfigurationFromJson(json);
}
