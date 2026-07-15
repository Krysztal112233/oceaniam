// ignore_for_file: invalid_annotation_target

import 'package:freezed_annotation/freezed_annotation.dart';

part 'configuration.freezed.dart';
part 'configuration.g.dart';

@freezed
class TokenConfiguration with _$TokenConfiguration {
  const factory TokenConfiguration({
    required String issuer,
    required List<String> audience,
  }) = _TokenConfiguration;

  factory TokenConfiguration.fromJson(Map<String, dynamic> json) =>
      _$TokenConfigurationFromJson(json);
}

@freezed
class Argon2Configuration with _$Argon2Configuration {
  const factory Argon2Configuration({
    @JsonKey(name: 'm_cost') required int mCost,
    @JsonKey(name: 't_cost') required int tCost,
    @JsonKey(name: 'p_cost') required int pCost,
  }) = _Argon2Configuration;

  factory Argon2Configuration.fromJson(Map<String, dynamic> json) =>
      _$Argon2ConfigurationFromJson(json);
}

@freezed
class PasswordConfiguration with _$PasswordConfiguration {
  const factory PasswordConfiguration({
    required Argon2Configuration argon2,
  }) = _PasswordConfiguration;

  factory PasswordConfiguration.fromJson(Map<String, dynamic> json) =>
      _$PasswordConfigurationFromJson(json);
}

@freezed
class AuthConfiguration with _$AuthConfiguration {
  const factory AuthConfiguration({
    required TokenConfiguration token,
    required PasswordConfiguration password,
  }) = _AuthConfiguration;

  factory AuthConfiguration.fromJson(Map<String, dynamic> json) =>
      _$AuthConfigurationFromJson(json);
}

@freezed
class RegistrationConfiguration with _$RegistrationConfiguration {
  const factory RegistrationConfiguration({required bool enabled}) =
      _RegistrationConfiguration;

  factory RegistrationConfiguration.fromJson(Map<String, dynamic> json) =>
      _$RegistrationConfigurationFromJson(json);
}

@freezed
class ApplicationConfiguration with _$ApplicationConfiguration {
  const factory ApplicationConfiguration({
    required AuthConfiguration auth,
    required RegistrationConfiguration registration,
  }) = _ApplicationConfiguration;

  factory ApplicationConfiguration.fromJson(Map<String, dynamic> json) =>
      _$ApplicationConfigurationFromJson(json);
}

/// Writable subset accepted by the application-configuration PATCH endpoint.
@freezed
class PatchTokenConfiguration with _$PatchTokenConfiguration {
  @JsonSerializable(includeIfNull: false, explicitToJson: true)
  const factory PatchTokenConfiguration({
    String? issuer,
    List<String>? audience,
  }) = _PatchTokenConfiguration;

  factory PatchTokenConfiguration.fromJson(Map<String, dynamic> json) =>
      _$PatchTokenConfigurationFromJson(json);
}

@freezed
class PatchAuthConfiguration with _$PatchAuthConfiguration {
  @JsonSerializable(includeIfNull: false, explicitToJson: true)
  const factory PatchAuthConfiguration({PatchTokenConfiguration? token}) =
      _PatchAuthConfiguration;

  factory PatchAuthConfiguration.fromJson(Map<String, dynamic> json) =>
      _$PatchAuthConfigurationFromJson(json);
}

@freezed
class PatchRegistrationConfiguration with _$PatchRegistrationConfiguration {
  @JsonSerializable(includeIfNull: false, explicitToJson: true)
  const factory PatchRegistrationConfiguration({bool? enabled}) =
      _PatchRegistrationConfiguration;

  factory PatchRegistrationConfiguration.fromJson(Map<String, dynamic> json) =>
      _$PatchRegistrationConfigurationFromJson(json);
}

@freezed
class PatchApplicationConfiguration with _$PatchApplicationConfiguration {
  @JsonSerializable(includeIfNull: false, explicitToJson: true)
  const factory PatchApplicationConfiguration({
    PatchAuthConfiguration? auth,
    PatchRegistrationConfiguration? registration,
  }) = _PatchApplicationConfiguration;

  factory PatchApplicationConfiguration.fromJson(Map<String, dynamic> json) =>
      _$PatchApplicationConfigurationFromJson(json);
}
