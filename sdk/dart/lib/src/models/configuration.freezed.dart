// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'configuration.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

AuthConfig _$AuthConfigFromJson(Map<String, dynamic> json) {
  return _AuthConfig.fromJson(json);
}

/// @nodoc
mixin _$AuthConfig {
  @JsonKey(name: 'token_issuer')
  String? get tokenIssuer => throw _privateConstructorUsedError;
  @JsonKey(name: 'token_audience')
  String? get tokenAudience => throw _privateConstructorUsedError;
  @JsonKey(name: 'password_policy')
  PasswordPolicy? get passwordPolicy => throw _privateConstructorUsedError;

  /// Serializes this AuthConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $AuthConfigCopyWith<AuthConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AuthConfigCopyWith<$Res> {
  factory $AuthConfigCopyWith(
          AuthConfig value, $Res Function(AuthConfig) then) =
      _$AuthConfigCopyWithImpl<$Res, AuthConfig>;
  @useResult
  $Res call(
      {@JsonKey(name: 'token_issuer') String? tokenIssuer,
      @JsonKey(name: 'token_audience') String? tokenAudience,
      @JsonKey(name: 'password_policy') PasswordPolicy? passwordPolicy});

  $PasswordPolicyCopyWith<$Res>? get passwordPolicy;
}

/// @nodoc
class _$AuthConfigCopyWithImpl<$Res, $Val extends AuthConfig>
    implements $AuthConfigCopyWith<$Res> {
  _$AuthConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? tokenIssuer = freezed,
    Object? tokenAudience = freezed,
    Object? passwordPolicy = freezed,
  }) {
    return _then(_value.copyWith(
      tokenIssuer: freezed == tokenIssuer
          ? _value.tokenIssuer
          : tokenIssuer // ignore: cast_nullable_to_non_nullable
              as String?,
      tokenAudience: freezed == tokenAudience
          ? _value.tokenAudience
          : tokenAudience // ignore: cast_nullable_to_non_nullable
              as String?,
      passwordPolicy: freezed == passwordPolicy
          ? _value.passwordPolicy
          : passwordPolicy // ignore: cast_nullable_to_non_nullable
              as PasswordPolicy?,
    ) as $Val);
  }

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $PasswordPolicyCopyWith<$Res>? get passwordPolicy {
    if (_value.passwordPolicy == null) {
      return null;
    }

    return $PasswordPolicyCopyWith<$Res>(_value.passwordPolicy!, (value) {
      return _then(_value.copyWith(passwordPolicy: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$AuthConfigImplCopyWith<$Res>
    implements $AuthConfigCopyWith<$Res> {
  factory _$$AuthConfigImplCopyWith(
          _$AuthConfigImpl value, $Res Function(_$AuthConfigImpl) then) =
      __$$AuthConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'token_issuer') String? tokenIssuer,
      @JsonKey(name: 'token_audience') String? tokenAudience,
      @JsonKey(name: 'password_policy') PasswordPolicy? passwordPolicy});

  @override
  $PasswordPolicyCopyWith<$Res>? get passwordPolicy;
}

/// @nodoc
class __$$AuthConfigImplCopyWithImpl<$Res>
    extends _$AuthConfigCopyWithImpl<$Res, _$AuthConfigImpl>
    implements _$$AuthConfigImplCopyWith<$Res> {
  __$$AuthConfigImplCopyWithImpl(
      _$AuthConfigImpl _value, $Res Function(_$AuthConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? tokenIssuer = freezed,
    Object? tokenAudience = freezed,
    Object? passwordPolicy = freezed,
  }) {
    return _then(_$AuthConfigImpl(
      tokenIssuer: freezed == tokenIssuer
          ? _value.tokenIssuer
          : tokenIssuer // ignore: cast_nullable_to_non_nullable
              as String?,
      tokenAudience: freezed == tokenAudience
          ? _value.tokenAudience
          : tokenAudience // ignore: cast_nullable_to_non_nullable
              as String?,
      passwordPolicy: freezed == passwordPolicy
          ? _value.passwordPolicy
          : passwordPolicy // ignore: cast_nullable_to_non_nullable
              as PasswordPolicy?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$AuthConfigImpl implements _AuthConfig {
  const _$AuthConfigImpl(
      {@JsonKey(name: 'token_issuer') this.tokenIssuer,
      @JsonKey(name: 'token_audience') this.tokenAudience,
      @JsonKey(name: 'password_policy') this.passwordPolicy});

  factory _$AuthConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$AuthConfigImplFromJson(json);

  @override
  @JsonKey(name: 'token_issuer')
  final String? tokenIssuer;
  @override
  @JsonKey(name: 'token_audience')
  final String? tokenAudience;
  @override
  @JsonKey(name: 'password_policy')
  final PasswordPolicy? passwordPolicy;

  @override
  String toString() {
    return 'AuthConfig(tokenIssuer: $tokenIssuer, tokenAudience: $tokenAudience, passwordPolicy: $passwordPolicy)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AuthConfigImpl &&
            (identical(other.tokenIssuer, tokenIssuer) ||
                other.tokenIssuer == tokenIssuer) &&
            (identical(other.tokenAudience, tokenAudience) ||
                other.tokenAudience == tokenAudience) &&
            (identical(other.passwordPolicy, passwordPolicy) ||
                other.passwordPolicy == passwordPolicy));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, tokenIssuer, tokenAudience, passwordPolicy);

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AuthConfigImplCopyWith<_$AuthConfigImpl> get copyWith =>
      __$$AuthConfigImplCopyWithImpl<_$AuthConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$AuthConfigImplToJson(
      this,
    );
  }
}

abstract class _AuthConfig implements AuthConfig {
  const factory _AuthConfig(
      {@JsonKey(name: 'token_issuer') final String? tokenIssuer,
      @JsonKey(name: 'token_audience') final String? tokenAudience,
      @JsonKey(name: 'password_policy')
      final PasswordPolicy? passwordPolicy}) = _$AuthConfigImpl;

  factory _AuthConfig.fromJson(Map<String, dynamic> json) =
      _$AuthConfigImpl.fromJson;

  @override
  @JsonKey(name: 'token_issuer')
  String? get tokenIssuer;
  @override
  @JsonKey(name: 'token_audience')
  String? get tokenAudience;
  @override
  @JsonKey(name: 'password_policy')
  PasswordPolicy? get passwordPolicy;

  /// Create a copy of AuthConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AuthConfigImplCopyWith<_$AuthConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

PasswordPolicy _$PasswordPolicyFromJson(Map<String, dynamic> json) {
  return _PasswordPolicy.fromJson(json);
}

/// @nodoc
mixin _$PasswordPolicy {
  @JsonKey(name: 'min_length')
  int? get minLength => throw _privateConstructorUsedError;
  @JsonKey(name: 'require_uppercase')
  bool? get requireUppercase => throw _privateConstructorUsedError;
  @JsonKey(name: 'require_lowercase')
  bool? get requireLowercase => throw _privateConstructorUsedError;
  @JsonKey(name: 'require_digit')
  bool? get requireDigit => throw _privateConstructorUsedError;
  @JsonKey(name: 'require_special')
  bool? get requireSpecial => throw _privateConstructorUsedError;

  /// Serializes this PasswordPolicy to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of PasswordPolicy
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $PasswordPolicyCopyWith<PasswordPolicy> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PasswordPolicyCopyWith<$Res> {
  factory $PasswordPolicyCopyWith(
          PasswordPolicy value, $Res Function(PasswordPolicy) then) =
      _$PasswordPolicyCopyWithImpl<$Res, PasswordPolicy>;
  @useResult
  $Res call(
      {@JsonKey(name: 'min_length') int? minLength,
      @JsonKey(name: 'require_uppercase') bool? requireUppercase,
      @JsonKey(name: 'require_lowercase') bool? requireLowercase,
      @JsonKey(name: 'require_digit') bool? requireDigit,
      @JsonKey(name: 'require_special') bool? requireSpecial});
}

/// @nodoc
class _$PasswordPolicyCopyWithImpl<$Res, $Val extends PasswordPolicy>
    implements $PasswordPolicyCopyWith<$Res> {
  _$PasswordPolicyCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PasswordPolicy
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? minLength = freezed,
    Object? requireUppercase = freezed,
    Object? requireLowercase = freezed,
    Object? requireDigit = freezed,
    Object? requireSpecial = freezed,
  }) {
    return _then(_value.copyWith(
      minLength: freezed == minLength
          ? _value.minLength
          : minLength // ignore: cast_nullable_to_non_nullable
              as int?,
      requireUppercase: freezed == requireUppercase
          ? _value.requireUppercase
          : requireUppercase // ignore: cast_nullable_to_non_nullable
              as bool?,
      requireLowercase: freezed == requireLowercase
          ? _value.requireLowercase
          : requireLowercase // ignore: cast_nullable_to_non_nullable
              as bool?,
      requireDigit: freezed == requireDigit
          ? _value.requireDigit
          : requireDigit // ignore: cast_nullable_to_non_nullable
              as bool?,
      requireSpecial: freezed == requireSpecial
          ? _value.requireSpecial
          : requireSpecial // ignore: cast_nullable_to_non_nullable
              as bool?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$PasswordPolicyImplCopyWith<$Res>
    implements $PasswordPolicyCopyWith<$Res> {
  factory _$$PasswordPolicyImplCopyWith(_$PasswordPolicyImpl value,
          $Res Function(_$PasswordPolicyImpl) then) =
      __$$PasswordPolicyImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'min_length') int? minLength,
      @JsonKey(name: 'require_uppercase') bool? requireUppercase,
      @JsonKey(name: 'require_lowercase') bool? requireLowercase,
      @JsonKey(name: 'require_digit') bool? requireDigit,
      @JsonKey(name: 'require_special') bool? requireSpecial});
}

/// @nodoc
class __$$PasswordPolicyImplCopyWithImpl<$Res>
    extends _$PasswordPolicyCopyWithImpl<$Res, _$PasswordPolicyImpl>
    implements _$$PasswordPolicyImplCopyWith<$Res> {
  __$$PasswordPolicyImplCopyWithImpl(
      _$PasswordPolicyImpl _value, $Res Function(_$PasswordPolicyImpl) _then)
      : super(_value, _then);

  /// Create a copy of PasswordPolicy
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? minLength = freezed,
    Object? requireUppercase = freezed,
    Object? requireLowercase = freezed,
    Object? requireDigit = freezed,
    Object? requireSpecial = freezed,
  }) {
    return _then(_$PasswordPolicyImpl(
      minLength: freezed == minLength
          ? _value.minLength
          : minLength // ignore: cast_nullable_to_non_nullable
              as int?,
      requireUppercase: freezed == requireUppercase
          ? _value.requireUppercase
          : requireUppercase // ignore: cast_nullable_to_non_nullable
              as bool?,
      requireLowercase: freezed == requireLowercase
          ? _value.requireLowercase
          : requireLowercase // ignore: cast_nullable_to_non_nullable
              as bool?,
      requireDigit: freezed == requireDigit
          ? _value.requireDigit
          : requireDigit // ignore: cast_nullable_to_non_nullable
              as bool?,
      requireSpecial: freezed == requireSpecial
          ? _value.requireSpecial
          : requireSpecial // ignore: cast_nullable_to_non_nullable
              as bool?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$PasswordPolicyImpl implements _PasswordPolicy {
  const _$PasswordPolicyImpl(
      {@JsonKey(name: 'min_length') this.minLength,
      @JsonKey(name: 'require_uppercase') this.requireUppercase,
      @JsonKey(name: 'require_lowercase') this.requireLowercase,
      @JsonKey(name: 'require_digit') this.requireDigit,
      @JsonKey(name: 'require_special') this.requireSpecial});

  factory _$PasswordPolicyImpl.fromJson(Map<String, dynamic> json) =>
      _$$PasswordPolicyImplFromJson(json);

  @override
  @JsonKey(name: 'min_length')
  final int? minLength;
  @override
  @JsonKey(name: 'require_uppercase')
  final bool? requireUppercase;
  @override
  @JsonKey(name: 'require_lowercase')
  final bool? requireLowercase;
  @override
  @JsonKey(name: 'require_digit')
  final bool? requireDigit;
  @override
  @JsonKey(name: 'require_special')
  final bool? requireSpecial;

  @override
  String toString() {
    return 'PasswordPolicy(minLength: $minLength, requireUppercase: $requireUppercase, requireLowercase: $requireLowercase, requireDigit: $requireDigit, requireSpecial: $requireSpecial)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PasswordPolicyImpl &&
            (identical(other.minLength, minLength) ||
                other.minLength == minLength) &&
            (identical(other.requireUppercase, requireUppercase) ||
                other.requireUppercase == requireUppercase) &&
            (identical(other.requireLowercase, requireLowercase) ||
                other.requireLowercase == requireLowercase) &&
            (identical(other.requireDigit, requireDigit) ||
                other.requireDigit == requireDigit) &&
            (identical(other.requireSpecial, requireSpecial) ||
                other.requireSpecial == requireSpecial));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, minLength, requireUppercase,
      requireLowercase, requireDigit, requireSpecial);

  /// Create a copy of PasswordPolicy
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PasswordPolicyImplCopyWith<_$PasswordPolicyImpl> get copyWith =>
      __$$PasswordPolicyImplCopyWithImpl<_$PasswordPolicyImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$PasswordPolicyImplToJson(
      this,
    );
  }
}

abstract class _PasswordPolicy implements PasswordPolicy {
  const factory _PasswordPolicy(
          {@JsonKey(name: 'min_length') final int? minLength,
          @JsonKey(name: 'require_uppercase') final bool? requireUppercase,
          @JsonKey(name: 'require_lowercase') final bool? requireLowercase,
          @JsonKey(name: 'require_digit') final bool? requireDigit,
          @JsonKey(name: 'require_special') final bool? requireSpecial}) =
      _$PasswordPolicyImpl;

  factory _PasswordPolicy.fromJson(Map<String, dynamic> json) =
      _$PasswordPolicyImpl.fromJson;

  @override
  @JsonKey(name: 'min_length')
  int? get minLength;
  @override
  @JsonKey(name: 'require_uppercase')
  bool? get requireUppercase;
  @override
  @JsonKey(name: 'require_lowercase')
  bool? get requireLowercase;
  @override
  @JsonKey(name: 'require_digit')
  bool? get requireDigit;
  @override
  @JsonKey(name: 'require_special')
  bool? get requireSpecial;

  /// Create a copy of PasswordPolicy
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PasswordPolicyImplCopyWith<_$PasswordPolicyImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

RegistrationConfig _$RegistrationConfigFromJson(Map<String, dynamic> json) {
  return _RegistrationConfig.fromJson(json);
}

/// @nodoc
mixin _$RegistrationConfig {
  bool? get enabled => throw _privateConstructorUsedError;

  /// Serializes this RegistrationConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of RegistrationConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $RegistrationConfigCopyWith<RegistrationConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $RegistrationConfigCopyWith<$Res> {
  factory $RegistrationConfigCopyWith(
          RegistrationConfig value, $Res Function(RegistrationConfig) then) =
      _$RegistrationConfigCopyWithImpl<$Res, RegistrationConfig>;
  @useResult
  $Res call({bool? enabled});
}

/// @nodoc
class _$RegistrationConfigCopyWithImpl<$Res, $Val extends RegistrationConfig>
    implements $RegistrationConfigCopyWith<$Res> {
  _$RegistrationConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of RegistrationConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? enabled = freezed,
  }) {
    return _then(_value.copyWith(
      enabled: freezed == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$RegistrationConfigImplCopyWith<$Res>
    implements $RegistrationConfigCopyWith<$Res> {
  factory _$$RegistrationConfigImplCopyWith(_$RegistrationConfigImpl value,
          $Res Function(_$RegistrationConfigImpl) then) =
      __$$RegistrationConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({bool? enabled});
}

/// @nodoc
class __$$RegistrationConfigImplCopyWithImpl<$Res>
    extends _$RegistrationConfigCopyWithImpl<$Res, _$RegistrationConfigImpl>
    implements _$$RegistrationConfigImplCopyWith<$Res> {
  __$$RegistrationConfigImplCopyWithImpl(_$RegistrationConfigImpl _value,
      $Res Function(_$RegistrationConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of RegistrationConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? enabled = freezed,
  }) {
    return _then(_$RegistrationConfigImpl(
      enabled: freezed == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$RegistrationConfigImpl implements _RegistrationConfig {
  const _$RegistrationConfigImpl({this.enabled});

  factory _$RegistrationConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$RegistrationConfigImplFromJson(json);

  @override
  final bool? enabled;

  @override
  String toString() {
    return 'RegistrationConfig(enabled: $enabled)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$RegistrationConfigImpl &&
            (identical(other.enabled, enabled) || other.enabled == enabled));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, enabled);

  /// Create a copy of RegistrationConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$RegistrationConfigImplCopyWith<_$RegistrationConfigImpl> get copyWith =>
      __$$RegistrationConfigImplCopyWithImpl<_$RegistrationConfigImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$RegistrationConfigImplToJson(
      this,
    );
  }
}

abstract class _RegistrationConfig implements RegistrationConfig {
  const factory _RegistrationConfig({final bool? enabled}) =
      _$RegistrationConfigImpl;

  factory _RegistrationConfig.fromJson(Map<String, dynamic> json) =
      _$RegistrationConfigImpl.fromJson;

  @override
  bool? get enabled;

  /// Create a copy of RegistrationConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$RegistrationConfigImplCopyWith<_$RegistrationConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

ApplicationConfiguration _$ApplicationConfigurationFromJson(
    Map<String, dynamic> json) {
  return _ApplicationConfiguration.fromJson(json);
}

/// @nodoc
mixin _$ApplicationConfiguration {
  AuthConfig? get auth => throw _privateConstructorUsedError;
  RegistrationConfig? get registration => throw _privateConstructorUsedError;

  /// Serializes this ApplicationConfiguration to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ApplicationConfigurationCopyWith<ApplicationConfiguration> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ApplicationConfigurationCopyWith<$Res> {
  factory $ApplicationConfigurationCopyWith(ApplicationConfiguration value,
          $Res Function(ApplicationConfiguration) then) =
      _$ApplicationConfigurationCopyWithImpl<$Res, ApplicationConfiguration>;
  @useResult
  $Res call({AuthConfig? auth, RegistrationConfig? registration});

  $AuthConfigCopyWith<$Res>? get auth;
  $RegistrationConfigCopyWith<$Res>? get registration;
}

/// @nodoc
class _$ApplicationConfigurationCopyWithImpl<$Res,
        $Val extends ApplicationConfiguration>
    implements $ApplicationConfigurationCopyWith<$Res> {
  _$ApplicationConfigurationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? auth = freezed,
    Object? registration = freezed,
  }) {
    return _then(_value.copyWith(
      auth: freezed == auth
          ? _value.auth
          : auth // ignore: cast_nullable_to_non_nullable
              as AuthConfig?,
      registration: freezed == registration
          ? _value.registration
          : registration // ignore: cast_nullable_to_non_nullable
              as RegistrationConfig?,
    ) as $Val);
  }

  /// Create a copy of ApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AuthConfigCopyWith<$Res>? get auth {
    if (_value.auth == null) {
      return null;
    }

    return $AuthConfigCopyWith<$Res>(_value.auth!, (value) {
      return _then(_value.copyWith(auth: value) as $Val);
    });
  }

  /// Create a copy of ApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $RegistrationConfigCopyWith<$Res>? get registration {
    if (_value.registration == null) {
      return null;
    }

    return $RegistrationConfigCopyWith<$Res>(_value.registration!, (value) {
      return _then(_value.copyWith(registration: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$ApplicationConfigurationImplCopyWith<$Res>
    implements $ApplicationConfigurationCopyWith<$Res> {
  factory _$$ApplicationConfigurationImplCopyWith(
          _$ApplicationConfigurationImpl value,
          $Res Function(_$ApplicationConfigurationImpl) then) =
      __$$ApplicationConfigurationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({AuthConfig? auth, RegistrationConfig? registration});

  @override
  $AuthConfigCopyWith<$Res>? get auth;
  @override
  $RegistrationConfigCopyWith<$Res>? get registration;
}

/// @nodoc
class __$$ApplicationConfigurationImplCopyWithImpl<$Res>
    extends _$ApplicationConfigurationCopyWithImpl<$Res,
        _$ApplicationConfigurationImpl>
    implements _$$ApplicationConfigurationImplCopyWith<$Res> {
  __$$ApplicationConfigurationImplCopyWithImpl(
      _$ApplicationConfigurationImpl _value,
      $Res Function(_$ApplicationConfigurationImpl) _then)
      : super(_value, _then);

  /// Create a copy of ApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? auth = freezed,
    Object? registration = freezed,
  }) {
    return _then(_$ApplicationConfigurationImpl(
      auth: freezed == auth
          ? _value.auth
          : auth // ignore: cast_nullable_to_non_nullable
              as AuthConfig?,
      registration: freezed == registration
          ? _value.registration
          : registration // ignore: cast_nullable_to_non_nullable
              as RegistrationConfig?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ApplicationConfigurationImpl implements _ApplicationConfiguration {
  const _$ApplicationConfigurationImpl({this.auth, this.registration});

  factory _$ApplicationConfigurationImpl.fromJson(Map<String, dynamic> json) =>
      _$$ApplicationConfigurationImplFromJson(json);

  @override
  final AuthConfig? auth;
  @override
  final RegistrationConfig? registration;

  @override
  String toString() {
    return 'ApplicationConfiguration(auth: $auth, registration: $registration)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ApplicationConfigurationImpl &&
            (identical(other.auth, auth) || other.auth == auth) &&
            (identical(other.registration, registration) ||
                other.registration == registration));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, auth, registration);

  /// Create a copy of ApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ApplicationConfigurationImplCopyWith<_$ApplicationConfigurationImpl>
      get copyWith => __$$ApplicationConfigurationImplCopyWithImpl<
          _$ApplicationConfigurationImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ApplicationConfigurationImplToJson(
      this,
    );
  }
}

abstract class _ApplicationConfiguration implements ApplicationConfiguration {
  const factory _ApplicationConfiguration(
      {final AuthConfig? auth,
      final RegistrationConfig? registration}) = _$ApplicationConfigurationImpl;

  factory _ApplicationConfiguration.fromJson(Map<String, dynamic> json) =
      _$ApplicationConfigurationImpl.fromJson;

  @override
  AuthConfig? get auth;
  @override
  RegistrationConfig? get registration;

  /// Create a copy of ApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ApplicationConfigurationImplCopyWith<_$ApplicationConfigurationImpl>
      get copyWith => throw _privateConstructorUsedError;
}
