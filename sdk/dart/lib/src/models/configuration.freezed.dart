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

TokenConfiguration _$TokenConfigurationFromJson(Map<String, dynamic> json) {
  return _TokenConfiguration.fromJson(json);
}

/// @nodoc
mixin _$TokenConfiguration {
  String get issuer => throw _privateConstructorUsedError;
  List<String> get audience => throw _privateConstructorUsedError;

  /// Serializes this TokenConfiguration to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of TokenConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $TokenConfigurationCopyWith<TokenConfiguration> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $TokenConfigurationCopyWith<$Res> {
  factory $TokenConfigurationCopyWith(
          TokenConfiguration value, $Res Function(TokenConfiguration) then) =
      _$TokenConfigurationCopyWithImpl<$Res, TokenConfiguration>;
  @useResult
  $Res call({String issuer, List<String> audience});
}

/// @nodoc
class _$TokenConfigurationCopyWithImpl<$Res, $Val extends TokenConfiguration>
    implements $TokenConfigurationCopyWith<$Res> {
  _$TokenConfigurationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of TokenConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? issuer = null,
    Object? audience = null,
  }) {
    return _then(_value.copyWith(
      issuer: null == issuer
          ? _value.issuer
          : issuer // ignore: cast_nullable_to_non_nullable
              as String,
      audience: null == audience
          ? _value.audience
          : audience // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$TokenConfigurationImplCopyWith<$Res>
    implements $TokenConfigurationCopyWith<$Res> {
  factory _$$TokenConfigurationImplCopyWith(_$TokenConfigurationImpl value,
          $Res Function(_$TokenConfigurationImpl) then) =
      __$$TokenConfigurationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String issuer, List<String> audience});
}

/// @nodoc
class __$$TokenConfigurationImplCopyWithImpl<$Res>
    extends _$TokenConfigurationCopyWithImpl<$Res, _$TokenConfigurationImpl>
    implements _$$TokenConfigurationImplCopyWith<$Res> {
  __$$TokenConfigurationImplCopyWithImpl(_$TokenConfigurationImpl _value,
      $Res Function(_$TokenConfigurationImpl) _then)
      : super(_value, _then);

  /// Create a copy of TokenConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? issuer = null,
    Object? audience = null,
  }) {
    return _then(_$TokenConfigurationImpl(
      issuer: null == issuer
          ? _value.issuer
          : issuer // ignore: cast_nullable_to_non_nullable
              as String,
      audience: null == audience
          ? _value._audience
          : audience // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$TokenConfigurationImpl implements _TokenConfiguration {
  const _$TokenConfigurationImpl(
      {required this.issuer, required final List<String> audience})
      : _audience = audience;

  factory _$TokenConfigurationImpl.fromJson(Map<String, dynamic> json) =>
      _$$TokenConfigurationImplFromJson(json);

  @override
  final String issuer;
  final List<String> _audience;
  @override
  List<String> get audience {
    if (_audience is EqualUnmodifiableListView) return _audience;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_audience);
  }

  @override
  String toString() {
    return 'TokenConfiguration(issuer: $issuer, audience: $audience)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$TokenConfigurationImpl &&
            (identical(other.issuer, issuer) || other.issuer == issuer) &&
            const DeepCollectionEquality().equals(other._audience, _audience));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType, issuer, const DeepCollectionEquality().hash(_audience));

  /// Create a copy of TokenConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$TokenConfigurationImplCopyWith<_$TokenConfigurationImpl> get copyWith =>
      __$$TokenConfigurationImplCopyWithImpl<_$TokenConfigurationImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$TokenConfigurationImplToJson(
      this,
    );
  }
}

abstract class _TokenConfiguration implements TokenConfiguration {
  const factory _TokenConfiguration(
      {required final String issuer,
      required final List<String> audience}) = _$TokenConfigurationImpl;

  factory _TokenConfiguration.fromJson(Map<String, dynamic> json) =
      _$TokenConfigurationImpl.fromJson;

  @override
  String get issuer;
  @override
  List<String> get audience;

  /// Create a copy of TokenConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$TokenConfigurationImplCopyWith<_$TokenConfigurationImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

Argon2Configuration _$Argon2ConfigurationFromJson(Map<String, dynamic> json) {
  return _Argon2Configuration.fromJson(json);
}

/// @nodoc
mixin _$Argon2Configuration {
  @JsonKey(name: 'm_cost')
  int get mCost => throw _privateConstructorUsedError;
  @JsonKey(name: 't_cost')
  int get tCost => throw _privateConstructorUsedError;
  @JsonKey(name: 'p_cost')
  int get pCost => throw _privateConstructorUsedError;

  /// Serializes this Argon2Configuration to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of Argon2Configuration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $Argon2ConfigurationCopyWith<Argon2Configuration> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $Argon2ConfigurationCopyWith<$Res> {
  factory $Argon2ConfigurationCopyWith(
          Argon2Configuration value, $Res Function(Argon2Configuration) then) =
      _$Argon2ConfigurationCopyWithImpl<$Res, Argon2Configuration>;
  @useResult
  $Res call(
      {@JsonKey(name: 'm_cost') int mCost,
      @JsonKey(name: 't_cost') int tCost,
      @JsonKey(name: 'p_cost') int pCost});
}

/// @nodoc
class _$Argon2ConfigurationCopyWithImpl<$Res, $Val extends Argon2Configuration>
    implements $Argon2ConfigurationCopyWith<$Res> {
  _$Argon2ConfigurationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Argon2Configuration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? mCost = null,
    Object? tCost = null,
    Object? pCost = null,
  }) {
    return _then(_value.copyWith(
      mCost: null == mCost
          ? _value.mCost
          : mCost // ignore: cast_nullable_to_non_nullable
              as int,
      tCost: null == tCost
          ? _value.tCost
          : tCost // ignore: cast_nullable_to_non_nullable
              as int,
      pCost: null == pCost
          ? _value.pCost
          : pCost // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$Argon2ConfigurationImplCopyWith<$Res>
    implements $Argon2ConfigurationCopyWith<$Res> {
  factory _$$Argon2ConfigurationImplCopyWith(_$Argon2ConfigurationImpl value,
          $Res Function(_$Argon2ConfigurationImpl) then) =
      __$$Argon2ConfigurationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'm_cost') int mCost,
      @JsonKey(name: 't_cost') int tCost,
      @JsonKey(name: 'p_cost') int pCost});
}

/// @nodoc
class __$$Argon2ConfigurationImplCopyWithImpl<$Res>
    extends _$Argon2ConfigurationCopyWithImpl<$Res, _$Argon2ConfigurationImpl>
    implements _$$Argon2ConfigurationImplCopyWith<$Res> {
  __$$Argon2ConfigurationImplCopyWithImpl(_$Argon2ConfigurationImpl _value,
      $Res Function(_$Argon2ConfigurationImpl) _then)
      : super(_value, _then);

  /// Create a copy of Argon2Configuration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? mCost = null,
    Object? tCost = null,
    Object? pCost = null,
  }) {
    return _then(_$Argon2ConfigurationImpl(
      mCost: null == mCost
          ? _value.mCost
          : mCost // ignore: cast_nullable_to_non_nullable
              as int,
      tCost: null == tCost
          ? _value.tCost
          : tCost // ignore: cast_nullable_to_non_nullable
              as int,
      pCost: null == pCost
          ? _value.pCost
          : pCost // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$Argon2ConfigurationImpl implements _Argon2Configuration {
  const _$Argon2ConfigurationImpl(
      {@JsonKey(name: 'm_cost') required this.mCost,
      @JsonKey(name: 't_cost') required this.tCost,
      @JsonKey(name: 'p_cost') required this.pCost});

  factory _$Argon2ConfigurationImpl.fromJson(Map<String, dynamic> json) =>
      _$$Argon2ConfigurationImplFromJson(json);

  @override
  @JsonKey(name: 'm_cost')
  final int mCost;
  @override
  @JsonKey(name: 't_cost')
  final int tCost;
  @override
  @JsonKey(name: 'p_cost')
  final int pCost;

  @override
  String toString() {
    return 'Argon2Configuration(mCost: $mCost, tCost: $tCost, pCost: $pCost)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Argon2ConfigurationImpl &&
            (identical(other.mCost, mCost) || other.mCost == mCost) &&
            (identical(other.tCost, tCost) || other.tCost == tCost) &&
            (identical(other.pCost, pCost) || other.pCost == pCost));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, mCost, tCost, pCost);

  /// Create a copy of Argon2Configuration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Argon2ConfigurationImplCopyWith<_$Argon2ConfigurationImpl> get copyWith =>
      __$$Argon2ConfigurationImplCopyWithImpl<_$Argon2ConfigurationImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$Argon2ConfigurationImplToJson(
      this,
    );
  }
}

abstract class _Argon2Configuration implements Argon2Configuration {
  const factory _Argon2Configuration(
          {@JsonKey(name: 'm_cost') required final int mCost,
          @JsonKey(name: 't_cost') required final int tCost,
          @JsonKey(name: 'p_cost') required final int pCost}) =
      _$Argon2ConfigurationImpl;

  factory _Argon2Configuration.fromJson(Map<String, dynamic> json) =
      _$Argon2ConfigurationImpl.fromJson;

  @override
  @JsonKey(name: 'm_cost')
  int get mCost;
  @override
  @JsonKey(name: 't_cost')
  int get tCost;
  @override
  @JsonKey(name: 'p_cost')
  int get pCost;

  /// Create a copy of Argon2Configuration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Argon2ConfigurationImplCopyWith<_$Argon2ConfigurationImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

PasswordConfiguration _$PasswordConfigurationFromJson(
    Map<String, dynamic> json) {
  return _PasswordConfiguration.fromJson(json);
}

/// @nodoc
mixin _$PasswordConfiguration {
  Argon2Configuration get argon2 => throw _privateConstructorUsedError;

  /// Serializes this PasswordConfiguration to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of PasswordConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $PasswordConfigurationCopyWith<PasswordConfiguration> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PasswordConfigurationCopyWith<$Res> {
  factory $PasswordConfigurationCopyWith(PasswordConfiguration value,
          $Res Function(PasswordConfiguration) then) =
      _$PasswordConfigurationCopyWithImpl<$Res, PasswordConfiguration>;
  @useResult
  $Res call({Argon2Configuration argon2});

  $Argon2ConfigurationCopyWith<$Res> get argon2;
}

/// @nodoc
class _$PasswordConfigurationCopyWithImpl<$Res,
        $Val extends PasswordConfiguration>
    implements $PasswordConfigurationCopyWith<$Res> {
  _$PasswordConfigurationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PasswordConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? argon2 = null,
  }) {
    return _then(_value.copyWith(
      argon2: null == argon2
          ? _value.argon2
          : argon2 // ignore: cast_nullable_to_non_nullable
              as Argon2Configuration,
    ) as $Val);
  }

  /// Create a copy of PasswordConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Argon2ConfigurationCopyWith<$Res> get argon2 {
    return $Argon2ConfigurationCopyWith<$Res>(_value.argon2, (value) {
      return _then(_value.copyWith(argon2: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$PasswordConfigurationImplCopyWith<$Res>
    implements $PasswordConfigurationCopyWith<$Res> {
  factory _$$PasswordConfigurationImplCopyWith(
          _$PasswordConfigurationImpl value,
          $Res Function(_$PasswordConfigurationImpl) then) =
      __$$PasswordConfigurationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({Argon2Configuration argon2});

  @override
  $Argon2ConfigurationCopyWith<$Res> get argon2;
}

/// @nodoc
class __$$PasswordConfigurationImplCopyWithImpl<$Res>
    extends _$PasswordConfigurationCopyWithImpl<$Res,
        _$PasswordConfigurationImpl>
    implements _$$PasswordConfigurationImplCopyWith<$Res> {
  __$$PasswordConfigurationImplCopyWithImpl(_$PasswordConfigurationImpl _value,
      $Res Function(_$PasswordConfigurationImpl) _then)
      : super(_value, _then);

  /// Create a copy of PasswordConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? argon2 = null,
  }) {
    return _then(_$PasswordConfigurationImpl(
      argon2: null == argon2
          ? _value.argon2
          : argon2 // ignore: cast_nullable_to_non_nullable
              as Argon2Configuration,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$PasswordConfigurationImpl implements _PasswordConfiguration {
  const _$PasswordConfigurationImpl({required this.argon2});

  factory _$PasswordConfigurationImpl.fromJson(Map<String, dynamic> json) =>
      _$$PasswordConfigurationImplFromJson(json);

  @override
  final Argon2Configuration argon2;

  @override
  String toString() {
    return 'PasswordConfiguration(argon2: $argon2)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PasswordConfigurationImpl &&
            (identical(other.argon2, argon2) || other.argon2 == argon2));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, argon2);

  /// Create a copy of PasswordConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PasswordConfigurationImplCopyWith<_$PasswordConfigurationImpl>
      get copyWith => __$$PasswordConfigurationImplCopyWithImpl<
          _$PasswordConfigurationImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$PasswordConfigurationImplToJson(
      this,
    );
  }
}

abstract class _PasswordConfiguration implements PasswordConfiguration {
  const factory _PasswordConfiguration(
          {required final Argon2Configuration argon2}) =
      _$PasswordConfigurationImpl;

  factory _PasswordConfiguration.fromJson(Map<String, dynamic> json) =
      _$PasswordConfigurationImpl.fromJson;

  @override
  Argon2Configuration get argon2;

  /// Create a copy of PasswordConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PasswordConfigurationImplCopyWith<_$PasswordConfigurationImpl>
      get copyWith => throw _privateConstructorUsedError;
}

AuthConfiguration _$AuthConfigurationFromJson(Map<String, dynamic> json) {
  return _AuthConfiguration.fromJson(json);
}

/// @nodoc
mixin _$AuthConfiguration {
  TokenConfiguration get token => throw _privateConstructorUsedError;
  PasswordConfiguration get password => throw _privateConstructorUsedError;

  /// Serializes this AuthConfiguration to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of AuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $AuthConfigurationCopyWith<AuthConfiguration> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AuthConfigurationCopyWith<$Res> {
  factory $AuthConfigurationCopyWith(
          AuthConfiguration value, $Res Function(AuthConfiguration) then) =
      _$AuthConfigurationCopyWithImpl<$Res, AuthConfiguration>;
  @useResult
  $Res call({TokenConfiguration token, PasswordConfiguration password});

  $TokenConfigurationCopyWith<$Res> get token;
  $PasswordConfigurationCopyWith<$Res> get password;
}

/// @nodoc
class _$AuthConfigurationCopyWithImpl<$Res, $Val extends AuthConfiguration>
    implements $AuthConfigurationCopyWith<$Res> {
  _$AuthConfigurationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of AuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? token = null,
    Object? password = null,
  }) {
    return _then(_value.copyWith(
      token: null == token
          ? _value.token
          : token // ignore: cast_nullable_to_non_nullable
              as TokenConfiguration,
      password: null == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as PasswordConfiguration,
    ) as $Val);
  }

  /// Create a copy of AuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $TokenConfigurationCopyWith<$Res> get token {
    return $TokenConfigurationCopyWith<$Res>(_value.token, (value) {
      return _then(_value.copyWith(token: value) as $Val);
    });
  }

  /// Create a copy of AuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $PasswordConfigurationCopyWith<$Res> get password {
    return $PasswordConfigurationCopyWith<$Res>(_value.password, (value) {
      return _then(_value.copyWith(password: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$AuthConfigurationImplCopyWith<$Res>
    implements $AuthConfigurationCopyWith<$Res> {
  factory _$$AuthConfigurationImplCopyWith(_$AuthConfigurationImpl value,
          $Res Function(_$AuthConfigurationImpl) then) =
      __$$AuthConfigurationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({TokenConfiguration token, PasswordConfiguration password});

  @override
  $TokenConfigurationCopyWith<$Res> get token;
  @override
  $PasswordConfigurationCopyWith<$Res> get password;
}

/// @nodoc
class __$$AuthConfigurationImplCopyWithImpl<$Res>
    extends _$AuthConfigurationCopyWithImpl<$Res, _$AuthConfigurationImpl>
    implements _$$AuthConfigurationImplCopyWith<$Res> {
  __$$AuthConfigurationImplCopyWithImpl(_$AuthConfigurationImpl _value,
      $Res Function(_$AuthConfigurationImpl) _then)
      : super(_value, _then);

  /// Create a copy of AuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? token = null,
    Object? password = null,
  }) {
    return _then(_$AuthConfigurationImpl(
      token: null == token
          ? _value.token
          : token // ignore: cast_nullable_to_non_nullable
              as TokenConfiguration,
      password: null == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as PasswordConfiguration,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$AuthConfigurationImpl implements _AuthConfiguration {
  const _$AuthConfigurationImpl({required this.token, required this.password});

  factory _$AuthConfigurationImpl.fromJson(Map<String, dynamic> json) =>
      _$$AuthConfigurationImplFromJson(json);

  @override
  final TokenConfiguration token;
  @override
  final PasswordConfiguration password;

  @override
  String toString() {
    return 'AuthConfiguration(token: $token, password: $password)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AuthConfigurationImpl &&
            (identical(other.token, token) || other.token == token) &&
            (identical(other.password, password) ||
                other.password == password));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, token, password);

  /// Create a copy of AuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AuthConfigurationImplCopyWith<_$AuthConfigurationImpl> get copyWith =>
      __$$AuthConfigurationImplCopyWithImpl<_$AuthConfigurationImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$AuthConfigurationImplToJson(
      this,
    );
  }
}

abstract class _AuthConfiguration implements AuthConfiguration {
  const factory _AuthConfiguration(
      {required final TokenConfiguration token,
      required final PasswordConfiguration password}) = _$AuthConfigurationImpl;

  factory _AuthConfiguration.fromJson(Map<String, dynamic> json) =
      _$AuthConfigurationImpl.fromJson;

  @override
  TokenConfiguration get token;
  @override
  PasswordConfiguration get password;

  /// Create a copy of AuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AuthConfigurationImplCopyWith<_$AuthConfigurationImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

RegistrationConfiguration _$RegistrationConfigurationFromJson(
    Map<String, dynamic> json) {
  return _RegistrationConfiguration.fromJson(json);
}

/// @nodoc
mixin _$RegistrationConfiguration {
  bool get enabled => throw _privateConstructorUsedError;

  /// Serializes this RegistrationConfiguration to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of RegistrationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $RegistrationConfigurationCopyWith<RegistrationConfiguration> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $RegistrationConfigurationCopyWith<$Res> {
  factory $RegistrationConfigurationCopyWith(RegistrationConfiguration value,
          $Res Function(RegistrationConfiguration) then) =
      _$RegistrationConfigurationCopyWithImpl<$Res, RegistrationConfiguration>;
  @useResult
  $Res call({bool enabled});
}

/// @nodoc
class _$RegistrationConfigurationCopyWithImpl<$Res,
        $Val extends RegistrationConfiguration>
    implements $RegistrationConfigurationCopyWith<$Res> {
  _$RegistrationConfigurationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of RegistrationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? enabled = null,
  }) {
    return _then(_value.copyWith(
      enabled: null == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$RegistrationConfigurationImplCopyWith<$Res>
    implements $RegistrationConfigurationCopyWith<$Res> {
  factory _$$RegistrationConfigurationImplCopyWith(
          _$RegistrationConfigurationImpl value,
          $Res Function(_$RegistrationConfigurationImpl) then) =
      __$$RegistrationConfigurationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({bool enabled});
}

/// @nodoc
class __$$RegistrationConfigurationImplCopyWithImpl<$Res>
    extends _$RegistrationConfigurationCopyWithImpl<$Res,
        _$RegistrationConfigurationImpl>
    implements _$$RegistrationConfigurationImplCopyWith<$Res> {
  __$$RegistrationConfigurationImplCopyWithImpl(
      _$RegistrationConfigurationImpl _value,
      $Res Function(_$RegistrationConfigurationImpl) _then)
      : super(_value, _then);

  /// Create a copy of RegistrationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? enabled = null,
  }) {
    return _then(_$RegistrationConfigurationImpl(
      enabled: null == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$RegistrationConfigurationImpl implements _RegistrationConfiguration {
  const _$RegistrationConfigurationImpl({required this.enabled});

  factory _$RegistrationConfigurationImpl.fromJson(Map<String, dynamic> json) =>
      _$$RegistrationConfigurationImplFromJson(json);

  @override
  final bool enabled;

  @override
  String toString() {
    return 'RegistrationConfiguration(enabled: $enabled)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$RegistrationConfigurationImpl &&
            (identical(other.enabled, enabled) || other.enabled == enabled));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, enabled);

  /// Create a copy of RegistrationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$RegistrationConfigurationImplCopyWith<_$RegistrationConfigurationImpl>
      get copyWith => __$$RegistrationConfigurationImplCopyWithImpl<
          _$RegistrationConfigurationImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$RegistrationConfigurationImplToJson(
      this,
    );
  }
}

abstract class _RegistrationConfiguration implements RegistrationConfiguration {
  const factory _RegistrationConfiguration({required final bool enabled}) =
      _$RegistrationConfigurationImpl;

  factory _RegistrationConfiguration.fromJson(Map<String, dynamic> json) =
      _$RegistrationConfigurationImpl.fromJson;

  @override
  bool get enabled;

  /// Create a copy of RegistrationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$RegistrationConfigurationImplCopyWith<_$RegistrationConfigurationImpl>
      get copyWith => throw _privateConstructorUsedError;
}

ApplicationConfiguration _$ApplicationConfigurationFromJson(
    Map<String, dynamic> json) {
  return _ApplicationConfiguration.fromJson(json);
}

/// @nodoc
mixin _$ApplicationConfiguration {
  AuthConfiguration get auth => throw _privateConstructorUsedError;
  RegistrationConfiguration get registration =>
      throw _privateConstructorUsedError;

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
  $Res call({AuthConfiguration auth, RegistrationConfiguration registration});

  $AuthConfigurationCopyWith<$Res> get auth;
  $RegistrationConfigurationCopyWith<$Res> get registration;
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
    Object? auth = null,
    Object? registration = null,
  }) {
    return _then(_value.copyWith(
      auth: null == auth
          ? _value.auth
          : auth // ignore: cast_nullable_to_non_nullable
              as AuthConfiguration,
      registration: null == registration
          ? _value.registration
          : registration // ignore: cast_nullable_to_non_nullable
              as RegistrationConfiguration,
    ) as $Val);
  }

  /// Create a copy of ApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AuthConfigurationCopyWith<$Res> get auth {
    return $AuthConfigurationCopyWith<$Res>(_value.auth, (value) {
      return _then(_value.copyWith(auth: value) as $Val);
    });
  }

  /// Create a copy of ApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $RegistrationConfigurationCopyWith<$Res> get registration {
    return $RegistrationConfigurationCopyWith<$Res>(_value.registration,
        (value) {
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
  $Res call({AuthConfiguration auth, RegistrationConfiguration registration});

  @override
  $AuthConfigurationCopyWith<$Res> get auth;
  @override
  $RegistrationConfigurationCopyWith<$Res> get registration;
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
    Object? auth = null,
    Object? registration = null,
  }) {
    return _then(_$ApplicationConfigurationImpl(
      auth: null == auth
          ? _value.auth
          : auth // ignore: cast_nullable_to_non_nullable
              as AuthConfiguration,
      registration: null == registration
          ? _value.registration
          : registration // ignore: cast_nullable_to_non_nullable
              as RegistrationConfiguration,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ApplicationConfigurationImpl implements _ApplicationConfiguration {
  const _$ApplicationConfigurationImpl(
      {required this.auth, required this.registration});

  factory _$ApplicationConfigurationImpl.fromJson(Map<String, dynamic> json) =>
      _$$ApplicationConfigurationImplFromJson(json);

  @override
  final AuthConfiguration auth;
  @override
  final RegistrationConfiguration registration;

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
          {required final AuthConfiguration auth,
          required final RegistrationConfiguration registration}) =
      _$ApplicationConfigurationImpl;

  factory _ApplicationConfiguration.fromJson(Map<String, dynamic> json) =
      _$ApplicationConfigurationImpl.fromJson;

  @override
  AuthConfiguration get auth;
  @override
  RegistrationConfiguration get registration;

  /// Create a copy of ApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ApplicationConfigurationImplCopyWith<_$ApplicationConfigurationImpl>
      get copyWith => throw _privateConstructorUsedError;
}

PatchTokenConfiguration _$PatchTokenConfigurationFromJson(
    Map<String, dynamic> json) {
  return _PatchTokenConfiguration.fromJson(json);
}

/// @nodoc
mixin _$PatchTokenConfiguration {
  String? get issuer => throw _privateConstructorUsedError;
  List<String>? get audience => throw _privateConstructorUsedError;

  /// Serializes this PatchTokenConfiguration to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of PatchTokenConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $PatchTokenConfigurationCopyWith<PatchTokenConfiguration> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PatchTokenConfigurationCopyWith<$Res> {
  factory $PatchTokenConfigurationCopyWith(PatchTokenConfiguration value,
          $Res Function(PatchTokenConfiguration) then) =
      _$PatchTokenConfigurationCopyWithImpl<$Res, PatchTokenConfiguration>;
  @useResult
  $Res call({String? issuer, List<String>? audience});
}

/// @nodoc
class _$PatchTokenConfigurationCopyWithImpl<$Res,
        $Val extends PatchTokenConfiguration>
    implements $PatchTokenConfigurationCopyWith<$Res> {
  _$PatchTokenConfigurationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PatchTokenConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? issuer = freezed,
    Object? audience = freezed,
  }) {
    return _then(_value.copyWith(
      issuer: freezed == issuer
          ? _value.issuer
          : issuer // ignore: cast_nullable_to_non_nullable
              as String?,
      audience: freezed == audience
          ? _value.audience
          : audience // ignore: cast_nullable_to_non_nullable
              as List<String>?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$PatchTokenConfigurationImplCopyWith<$Res>
    implements $PatchTokenConfigurationCopyWith<$Res> {
  factory _$$PatchTokenConfigurationImplCopyWith(
          _$PatchTokenConfigurationImpl value,
          $Res Function(_$PatchTokenConfigurationImpl) then) =
      __$$PatchTokenConfigurationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String? issuer, List<String>? audience});
}

/// @nodoc
class __$$PatchTokenConfigurationImplCopyWithImpl<$Res>
    extends _$PatchTokenConfigurationCopyWithImpl<$Res,
        _$PatchTokenConfigurationImpl>
    implements _$$PatchTokenConfigurationImplCopyWith<$Res> {
  __$$PatchTokenConfigurationImplCopyWithImpl(
      _$PatchTokenConfigurationImpl _value,
      $Res Function(_$PatchTokenConfigurationImpl) _then)
      : super(_value, _then);

  /// Create a copy of PatchTokenConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? issuer = freezed,
    Object? audience = freezed,
  }) {
    return _then(_$PatchTokenConfigurationImpl(
      issuer: freezed == issuer
          ? _value.issuer
          : issuer // ignore: cast_nullable_to_non_nullable
              as String?,
      audience: freezed == audience
          ? _value._audience
          : audience // ignore: cast_nullable_to_non_nullable
              as List<String>?,
    ));
  }
}

/// @nodoc

@JsonSerializable(includeIfNull: false, explicitToJson: true)
class _$PatchTokenConfigurationImpl implements _PatchTokenConfiguration {
  const _$PatchTokenConfigurationImpl(
      {this.issuer, final List<String>? audience})
      : _audience = audience;

  factory _$PatchTokenConfigurationImpl.fromJson(Map<String, dynamic> json) =>
      _$$PatchTokenConfigurationImplFromJson(json);

  @override
  final String? issuer;
  final List<String>? _audience;
  @override
  List<String>? get audience {
    final value = _audience;
    if (value == null) return null;
    if (_audience is EqualUnmodifiableListView) return _audience;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(value);
  }

  @override
  String toString() {
    return 'PatchTokenConfiguration(issuer: $issuer, audience: $audience)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PatchTokenConfigurationImpl &&
            (identical(other.issuer, issuer) || other.issuer == issuer) &&
            const DeepCollectionEquality().equals(other._audience, _audience));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType, issuer, const DeepCollectionEquality().hash(_audience));

  /// Create a copy of PatchTokenConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PatchTokenConfigurationImplCopyWith<_$PatchTokenConfigurationImpl>
      get copyWith => __$$PatchTokenConfigurationImplCopyWithImpl<
          _$PatchTokenConfigurationImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$PatchTokenConfigurationImplToJson(
      this,
    );
  }
}

abstract class _PatchTokenConfiguration implements PatchTokenConfiguration {
  const factory _PatchTokenConfiguration(
      {final String? issuer,
      final List<String>? audience}) = _$PatchTokenConfigurationImpl;

  factory _PatchTokenConfiguration.fromJson(Map<String, dynamic> json) =
      _$PatchTokenConfigurationImpl.fromJson;

  @override
  String? get issuer;
  @override
  List<String>? get audience;

  /// Create a copy of PatchTokenConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PatchTokenConfigurationImplCopyWith<_$PatchTokenConfigurationImpl>
      get copyWith => throw _privateConstructorUsedError;
}

PatchAuthConfiguration _$PatchAuthConfigurationFromJson(
    Map<String, dynamic> json) {
  return _PatchAuthConfiguration.fromJson(json);
}

/// @nodoc
mixin _$PatchAuthConfiguration {
  PatchTokenConfiguration? get token => throw _privateConstructorUsedError;

  /// Serializes this PatchAuthConfiguration to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of PatchAuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $PatchAuthConfigurationCopyWith<PatchAuthConfiguration> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PatchAuthConfigurationCopyWith<$Res> {
  factory $PatchAuthConfigurationCopyWith(PatchAuthConfiguration value,
          $Res Function(PatchAuthConfiguration) then) =
      _$PatchAuthConfigurationCopyWithImpl<$Res, PatchAuthConfiguration>;
  @useResult
  $Res call({PatchTokenConfiguration? token});

  $PatchTokenConfigurationCopyWith<$Res>? get token;
}

/// @nodoc
class _$PatchAuthConfigurationCopyWithImpl<$Res,
        $Val extends PatchAuthConfiguration>
    implements $PatchAuthConfigurationCopyWith<$Res> {
  _$PatchAuthConfigurationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PatchAuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? token = freezed,
  }) {
    return _then(_value.copyWith(
      token: freezed == token
          ? _value.token
          : token // ignore: cast_nullable_to_non_nullable
              as PatchTokenConfiguration?,
    ) as $Val);
  }

  /// Create a copy of PatchAuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $PatchTokenConfigurationCopyWith<$Res>? get token {
    if (_value.token == null) {
      return null;
    }

    return $PatchTokenConfigurationCopyWith<$Res>(_value.token!, (value) {
      return _then(_value.copyWith(token: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$PatchAuthConfigurationImplCopyWith<$Res>
    implements $PatchAuthConfigurationCopyWith<$Res> {
  factory _$$PatchAuthConfigurationImplCopyWith(
          _$PatchAuthConfigurationImpl value,
          $Res Function(_$PatchAuthConfigurationImpl) then) =
      __$$PatchAuthConfigurationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({PatchTokenConfiguration? token});

  @override
  $PatchTokenConfigurationCopyWith<$Res>? get token;
}

/// @nodoc
class __$$PatchAuthConfigurationImplCopyWithImpl<$Res>
    extends _$PatchAuthConfigurationCopyWithImpl<$Res,
        _$PatchAuthConfigurationImpl>
    implements _$$PatchAuthConfigurationImplCopyWith<$Res> {
  __$$PatchAuthConfigurationImplCopyWithImpl(
      _$PatchAuthConfigurationImpl _value,
      $Res Function(_$PatchAuthConfigurationImpl) _then)
      : super(_value, _then);

  /// Create a copy of PatchAuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? token = freezed,
  }) {
    return _then(_$PatchAuthConfigurationImpl(
      token: freezed == token
          ? _value.token
          : token // ignore: cast_nullable_to_non_nullable
              as PatchTokenConfiguration?,
    ));
  }
}

/// @nodoc

@JsonSerializable(includeIfNull: false, explicitToJson: true)
class _$PatchAuthConfigurationImpl implements _PatchAuthConfiguration {
  const _$PatchAuthConfigurationImpl({this.token});

  factory _$PatchAuthConfigurationImpl.fromJson(Map<String, dynamic> json) =>
      _$$PatchAuthConfigurationImplFromJson(json);

  @override
  final PatchTokenConfiguration? token;

  @override
  String toString() {
    return 'PatchAuthConfiguration(token: $token)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PatchAuthConfigurationImpl &&
            (identical(other.token, token) || other.token == token));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, token);

  /// Create a copy of PatchAuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PatchAuthConfigurationImplCopyWith<_$PatchAuthConfigurationImpl>
      get copyWith => __$$PatchAuthConfigurationImplCopyWithImpl<
          _$PatchAuthConfigurationImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$PatchAuthConfigurationImplToJson(
      this,
    );
  }
}

abstract class _PatchAuthConfiguration implements PatchAuthConfiguration {
  const factory _PatchAuthConfiguration(
      {final PatchTokenConfiguration? token}) = _$PatchAuthConfigurationImpl;

  factory _PatchAuthConfiguration.fromJson(Map<String, dynamic> json) =
      _$PatchAuthConfigurationImpl.fromJson;

  @override
  PatchTokenConfiguration? get token;

  /// Create a copy of PatchAuthConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PatchAuthConfigurationImplCopyWith<_$PatchAuthConfigurationImpl>
      get copyWith => throw _privateConstructorUsedError;
}

PatchRegistrationConfiguration _$PatchRegistrationConfigurationFromJson(
    Map<String, dynamic> json) {
  return _PatchRegistrationConfiguration.fromJson(json);
}

/// @nodoc
mixin _$PatchRegistrationConfiguration {
  bool? get enabled => throw _privateConstructorUsedError;

  /// Serializes this PatchRegistrationConfiguration to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of PatchRegistrationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $PatchRegistrationConfigurationCopyWith<PatchRegistrationConfiguration>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PatchRegistrationConfigurationCopyWith<$Res> {
  factory $PatchRegistrationConfigurationCopyWith(
          PatchRegistrationConfiguration value,
          $Res Function(PatchRegistrationConfiguration) then) =
      _$PatchRegistrationConfigurationCopyWithImpl<$Res,
          PatchRegistrationConfiguration>;
  @useResult
  $Res call({bool? enabled});
}

/// @nodoc
class _$PatchRegistrationConfigurationCopyWithImpl<$Res,
        $Val extends PatchRegistrationConfiguration>
    implements $PatchRegistrationConfigurationCopyWith<$Res> {
  _$PatchRegistrationConfigurationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PatchRegistrationConfiguration
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
abstract class _$$PatchRegistrationConfigurationImplCopyWith<$Res>
    implements $PatchRegistrationConfigurationCopyWith<$Res> {
  factory _$$PatchRegistrationConfigurationImplCopyWith(
          _$PatchRegistrationConfigurationImpl value,
          $Res Function(_$PatchRegistrationConfigurationImpl) then) =
      __$$PatchRegistrationConfigurationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({bool? enabled});
}

/// @nodoc
class __$$PatchRegistrationConfigurationImplCopyWithImpl<$Res>
    extends _$PatchRegistrationConfigurationCopyWithImpl<$Res,
        _$PatchRegistrationConfigurationImpl>
    implements _$$PatchRegistrationConfigurationImplCopyWith<$Res> {
  __$$PatchRegistrationConfigurationImplCopyWithImpl(
      _$PatchRegistrationConfigurationImpl _value,
      $Res Function(_$PatchRegistrationConfigurationImpl) _then)
      : super(_value, _then);

  /// Create a copy of PatchRegistrationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? enabled = freezed,
  }) {
    return _then(_$PatchRegistrationConfigurationImpl(
      enabled: freezed == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool?,
    ));
  }
}

/// @nodoc

@JsonSerializable(includeIfNull: false, explicitToJson: true)
class _$PatchRegistrationConfigurationImpl
    implements _PatchRegistrationConfiguration {
  const _$PatchRegistrationConfigurationImpl({this.enabled});

  factory _$PatchRegistrationConfigurationImpl.fromJson(
          Map<String, dynamic> json) =>
      _$$PatchRegistrationConfigurationImplFromJson(json);

  @override
  final bool? enabled;

  @override
  String toString() {
    return 'PatchRegistrationConfiguration(enabled: $enabled)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PatchRegistrationConfigurationImpl &&
            (identical(other.enabled, enabled) || other.enabled == enabled));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, enabled);

  /// Create a copy of PatchRegistrationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PatchRegistrationConfigurationImplCopyWith<
          _$PatchRegistrationConfigurationImpl>
      get copyWith => __$$PatchRegistrationConfigurationImplCopyWithImpl<
          _$PatchRegistrationConfigurationImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$PatchRegistrationConfigurationImplToJson(
      this,
    );
  }
}

abstract class _PatchRegistrationConfiguration
    implements PatchRegistrationConfiguration {
  const factory _PatchRegistrationConfiguration({final bool? enabled}) =
      _$PatchRegistrationConfigurationImpl;

  factory _PatchRegistrationConfiguration.fromJson(Map<String, dynamic> json) =
      _$PatchRegistrationConfigurationImpl.fromJson;

  @override
  bool? get enabled;

  /// Create a copy of PatchRegistrationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PatchRegistrationConfigurationImplCopyWith<
          _$PatchRegistrationConfigurationImpl>
      get copyWith => throw _privateConstructorUsedError;
}

PatchApplicationConfiguration _$PatchApplicationConfigurationFromJson(
    Map<String, dynamic> json) {
  return _PatchApplicationConfiguration.fromJson(json);
}

/// @nodoc
mixin _$PatchApplicationConfiguration {
  PatchAuthConfiguration? get auth => throw _privateConstructorUsedError;
  PatchRegistrationConfiguration? get registration =>
      throw _privateConstructorUsedError;

  /// Serializes this PatchApplicationConfiguration to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of PatchApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $PatchApplicationConfigurationCopyWith<PatchApplicationConfiguration>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PatchApplicationConfigurationCopyWith<$Res> {
  factory $PatchApplicationConfigurationCopyWith(
          PatchApplicationConfiguration value,
          $Res Function(PatchApplicationConfiguration) then) =
      _$PatchApplicationConfigurationCopyWithImpl<$Res,
          PatchApplicationConfiguration>;
  @useResult
  $Res call(
      {PatchAuthConfiguration? auth,
      PatchRegistrationConfiguration? registration});

  $PatchAuthConfigurationCopyWith<$Res>? get auth;
  $PatchRegistrationConfigurationCopyWith<$Res>? get registration;
}

/// @nodoc
class _$PatchApplicationConfigurationCopyWithImpl<$Res,
        $Val extends PatchApplicationConfiguration>
    implements $PatchApplicationConfigurationCopyWith<$Res> {
  _$PatchApplicationConfigurationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PatchApplicationConfiguration
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
              as PatchAuthConfiguration?,
      registration: freezed == registration
          ? _value.registration
          : registration // ignore: cast_nullable_to_non_nullable
              as PatchRegistrationConfiguration?,
    ) as $Val);
  }

  /// Create a copy of PatchApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $PatchAuthConfigurationCopyWith<$Res>? get auth {
    if (_value.auth == null) {
      return null;
    }

    return $PatchAuthConfigurationCopyWith<$Res>(_value.auth!, (value) {
      return _then(_value.copyWith(auth: value) as $Val);
    });
  }

  /// Create a copy of PatchApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $PatchRegistrationConfigurationCopyWith<$Res>? get registration {
    if (_value.registration == null) {
      return null;
    }

    return $PatchRegistrationConfigurationCopyWith<$Res>(_value.registration!,
        (value) {
      return _then(_value.copyWith(registration: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$PatchApplicationConfigurationImplCopyWith<$Res>
    implements $PatchApplicationConfigurationCopyWith<$Res> {
  factory _$$PatchApplicationConfigurationImplCopyWith(
          _$PatchApplicationConfigurationImpl value,
          $Res Function(_$PatchApplicationConfigurationImpl) then) =
      __$$PatchApplicationConfigurationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {PatchAuthConfiguration? auth,
      PatchRegistrationConfiguration? registration});

  @override
  $PatchAuthConfigurationCopyWith<$Res>? get auth;
  @override
  $PatchRegistrationConfigurationCopyWith<$Res>? get registration;
}

/// @nodoc
class __$$PatchApplicationConfigurationImplCopyWithImpl<$Res>
    extends _$PatchApplicationConfigurationCopyWithImpl<$Res,
        _$PatchApplicationConfigurationImpl>
    implements _$$PatchApplicationConfigurationImplCopyWith<$Res> {
  __$$PatchApplicationConfigurationImplCopyWithImpl(
      _$PatchApplicationConfigurationImpl _value,
      $Res Function(_$PatchApplicationConfigurationImpl) _then)
      : super(_value, _then);

  /// Create a copy of PatchApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? auth = freezed,
    Object? registration = freezed,
  }) {
    return _then(_$PatchApplicationConfigurationImpl(
      auth: freezed == auth
          ? _value.auth
          : auth // ignore: cast_nullable_to_non_nullable
              as PatchAuthConfiguration?,
      registration: freezed == registration
          ? _value.registration
          : registration // ignore: cast_nullable_to_non_nullable
              as PatchRegistrationConfiguration?,
    ));
  }
}

/// @nodoc

@JsonSerializable(includeIfNull: false, explicitToJson: true)
class _$PatchApplicationConfigurationImpl
    implements _PatchApplicationConfiguration {
  const _$PatchApplicationConfigurationImpl({this.auth, this.registration});

  factory _$PatchApplicationConfigurationImpl.fromJson(
          Map<String, dynamic> json) =>
      _$$PatchApplicationConfigurationImplFromJson(json);

  @override
  final PatchAuthConfiguration? auth;
  @override
  final PatchRegistrationConfiguration? registration;

  @override
  String toString() {
    return 'PatchApplicationConfiguration(auth: $auth, registration: $registration)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PatchApplicationConfigurationImpl &&
            (identical(other.auth, auth) || other.auth == auth) &&
            (identical(other.registration, registration) ||
                other.registration == registration));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, auth, registration);

  /// Create a copy of PatchApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PatchApplicationConfigurationImplCopyWith<
          _$PatchApplicationConfigurationImpl>
      get copyWith => __$$PatchApplicationConfigurationImplCopyWithImpl<
          _$PatchApplicationConfigurationImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$PatchApplicationConfigurationImplToJson(
      this,
    );
  }
}

abstract class _PatchApplicationConfiguration
    implements PatchApplicationConfiguration {
  const factory _PatchApplicationConfiguration(
          {final PatchAuthConfiguration? auth,
          final PatchRegistrationConfiguration? registration}) =
      _$PatchApplicationConfigurationImpl;

  factory _PatchApplicationConfiguration.fromJson(Map<String, dynamic> json) =
      _$PatchApplicationConfigurationImpl.fromJson;

  @override
  PatchAuthConfiguration? get auth;
  @override
  PatchRegistrationConfiguration? get registration;

  /// Create a copy of PatchApplicationConfiguration
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PatchApplicationConfigurationImplCopyWith<
          _$PatchApplicationConfigurationImpl>
      get copyWith => throw _privateConstructorUsedError;
}
