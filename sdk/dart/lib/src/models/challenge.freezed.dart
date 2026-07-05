// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'challenge.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

ApplicationChallenge _$ApplicationChallengeFromJson(Map<String, dynamic> json) {
  return _ApplicationChallenge.fromJson(json);
}

/// @nodoc
mixin _$ApplicationChallenge {
  String get id => throw _privateConstructorUsedError;
  @JsonKey(name: 'factor_type')
  String get factorType => throw _privateConstructorUsedError;
  String get purpose => throw _privateConstructorUsedError;
  String get status => throw _privateConstructorUsedError;
  @JsonKey(name: 'expires_at')
  String get expiresAt => throw _privateConstructorUsedError;

  /// Serializes this ApplicationChallenge to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ApplicationChallenge
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ApplicationChallengeCopyWith<ApplicationChallenge> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ApplicationChallengeCopyWith<$Res> {
  factory $ApplicationChallengeCopyWith(ApplicationChallenge value,
          $Res Function(ApplicationChallenge) then) =
      _$ApplicationChallengeCopyWithImpl<$Res, ApplicationChallenge>;
  @useResult
  $Res call(
      {String id,
      @JsonKey(name: 'factor_type') String factorType,
      String purpose,
      String status,
      @JsonKey(name: 'expires_at') String expiresAt});
}

/// @nodoc
class _$ApplicationChallengeCopyWithImpl<$Res,
        $Val extends ApplicationChallenge>
    implements $ApplicationChallengeCopyWith<$Res> {
  _$ApplicationChallengeCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ApplicationChallenge
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? factorType = null,
    Object? purpose = null,
    Object? status = null,
    Object? expiresAt = null,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      factorType: null == factorType
          ? _value.factorType
          : factorType // ignore: cast_nullable_to_non_nullable
              as String,
      purpose: null == purpose
          ? _value.purpose
          : purpose // ignore: cast_nullable_to_non_nullable
              as String,
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as String,
      expiresAt: null == expiresAt
          ? _value.expiresAt
          : expiresAt // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ApplicationChallengeImplCopyWith<$Res>
    implements $ApplicationChallengeCopyWith<$Res> {
  factory _$$ApplicationChallengeImplCopyWith(_$ApplicationChallengeImpl value,
          $Res Function(_$ApplicationChallengeImpl) then) =
      __$$ApplicationChallengeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String id,
      @JsonKey(name: 'factor_type') String factorType,
      String purpose,
      String status,
      @JsonKey(name: 'expires_at') String expiresAt});
}

/// @nodoc
class __$$ApplicationChallengeImplCopyWithImpl<$Res>
    extends _$ApplicationChallengeCopyWithImpl<$Res, _$ApplicationChallengeImpl>
    implements _$$ApplicationChallengeImplCopyWith<$Res> {
  __$$ApplicationChallengeImplCopyWithImpl(_$ApplicationChallengeImpl _value,
      $Res Function(_$ApplicationChallengeImpl) _then)
      : super(_value, _then);

  /// Create a copy of ApplicationChallenge
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? factorType = null,
    Object? purpose = null,
    Object? status = null,
    Object? expiresAt = null,
  }) {
    return _then(_$ApplicationChallengeImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      factorType: null == factorType
          ? _value.factorType
          : factorType // ignore: cast_nullable_to_non_nullable
              as String,
      purpose: null == purpose
          ? _value.purpose
          : purpose // ignore: cast_nullable_to_non_nullable
              as String,
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as String,
      expiresAt: null == expiresAt
          ? _value.expiresAt
          : expiresAt // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ApplicationChallengeImpl implements _ApplicationChallenge {
  const _$ApplicationChallengeImpl(
      {required this.id,
      @JsonKey(name: 'factor_type') required this.factorType,
      required this.purpose,
      required this.status,
      @JsonKey(name: 'expires_at') required this.expiresAt});

  factory _$ApplicationChallengeImpl.fromJson(Map<String, dynamic> json) =>
      _$$ApplicationChallengeImplFromJson(json);

  @override
  final String id;
  @override
  @JsonKey(name: 'factor_type')
  final String factorType;
  @override
  final String purpose;
  @override
  final String status;
  @override
  @JsonKey(name: 'expires_at')
  final String expiresAt;

  @override
  String toString() {
    return 'ApplicationChallenge(id: $id, factorType: $factorType, purpose: $purpose, status: $status, expiresAt: $expiresAt)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ApplicationChallengeImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.factorType, factorType) ||
                other.factorType == factorType) &&
            (identical(other.purpose, purpose) || other.purpose == purpose) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.expiresAt, expiresAt) ||
                other.expiresAt == expiresAt));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, id, factorType, purpose, status, expiresAt);

  /// Create a copy of ApplicationChallenge
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ApplicationChallengeImplCopyWith<_$ApplicationChallengeImpl>
      get copyWith =>
          __$$ApplicationChallengeImplCopyWithImpl<_$ApplicationChallengeImpl>(
              this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ApplicationChallengeImplToJson(
      this,
    );
  }
}

abstract class _ApplicationChallenge implements ApplicationChallenge {
  const factory _ApplicationChallenge(
          {required final String id,
          @JsonKey(name: 'factor_type') required final String factorType,
          required final String purpose,
          required final String status,
          @JsonKey(name: 'expires_at') required final String expiresAt}) =
      _$ApplicationChallengeImpl;

  factory _ApplicationChallenge.fromJson(Map<String, dynamic> json) =
      _$ApplicationChallengeImpl.fromJson;

  @override
  String get id;
  @override
  @JsonKey(name: 'factor_type')
  String get factorType;
  @override
  String get purpose;
  @override
  String get status;
  @override
  @JsonKey(name: 'expires_at')
  String get expiresAt;

  /// Create a copy of ApplicationChallenge
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ApplicationChallengeImplCopyWith<_$ApplicationChallengeImpl>
      get copyWith => throw _privateConstructorUsedError;
}

SigninChallenge _$SigninChallengeFromJson(Map<String, dynamic> json) {
  return _SigninChallenge.fromJson(json);
}

/// @nodoc
mixin _$SigninChallenge {
  @JsonKey(name: 'challenge_id')
  String get challengeId => throw _privateConstructorUsedError;
  @JsonKey(name: 'factor_type')
  String get factorType => throw _privateConstructorUsedError;
  @JsonKey(name: 'expires_at')
  String get expiresAt => throw _privateConstructorUsedError;

  /// Serializes this SigninChallenge to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of SigninChallenge
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $SigninChallengeCopyWith<SigninChallenge> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SigninChallengeCopyWith<$Res> {
  factory $SigninChallengeCopyWith(
          SigninChallenge value, $Res Function(SigninChallenge) then) =
      _$SigninChallengeCopyWithImpl<$Res, SigninChallenge>;
  @useResult
  $Res call(
      {@JsonKey(name: 'challenge_id') String challengeId,
      @JsonKey(name: 'factor_type') String factorType,
      @JsonKey(name: 'expires_at') String expiresAt});
}

/// @nodoc
class _$SigninChallengeCopyWithImpl<$Res, $Val extends SigninChallenge>
    implements $SigninChallengeCopyWith<$Res> {
  _$SigninChallengeCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of SigninChallenge
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? challengeId = null,
    Object? factorType = null,
    Object? expiresAt = null,
  }) {
    return _then(_value.copyWith(
      challengeId: null == challengeId
          ? _value.challengeId
          : challengeId // ignore: cast_nullable_to_non_nullable
              as String,
      factorType: null == factorType
          ? _value.factorType
          : factorType // ignore: cast_nullable_to_non_nullable
              as String,
      expiresAt: null == expiresAt
          ? _value.expiresAt
          : expiresAt // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SigninChallengeImplCopyWith<$Res>
    implements $SigninChallengeCopyWith<$Res> {
  factory _$$SigninChallengeImplCopyWith(_$SigninChallengeImpl value,
          $Res Function(_$SigninChallengeImpl) then) =
      __$$SigninChallengeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'challenge_id') String challengeId,
      @JsonKey(name: 'factor_type') String factorType,
      @JsonKey(name: 'expires_at') String expiresAt});
}

/// @nodoc
class __$$SigninChallengeImplCopyWithImpl<$Res>
    extends _$SigninChallengeCopyWithImpl<$Res, _$SigninChallengeImpl>
    implements _$$SigninChallengeImplCopyWith<$Res> {
  __$$SigninChallengeImplCopyWithImpl(
      _$SigninChallengeImpl _value, $Res Function(_$SigninChallengeImpl) _then)
      : super(_value, _then);

  /// Create a copy of SigninChallenge
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? challengeId = null,
    Object? factorType = null,
    Object? expiresAt = null,
  }) {
    return _then(_$SigninChallengeImpl(
      challengeId: null == challengeId
          ? _value.challengeId
          : challengeId // ignore: cast_nullable_to_non_nullable
              as String,
      factorType: null == factorType
          ? _value.factorType
          : factorType // ignore: cast_nullable_to_non_nullable
              as String,
      expiresAt: null == expiresAt
          ? _value.expiresAt
          : expiresAt // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SigninChallengeImpl implements _SigninChallenge {
  const _$SigninChallengeImpl(
      {@JsonKey(name: 'challenge_id') required this.challengeId,
      @JsonKey(name: 'factor_type') required this.factorType,
      @JsonKey(name: 'expires_at') required this.expiresAt});

  factory _$SigninChallengeImpl.fromJson(Map<String, dynamic> json) =>
      _$$SigninChallengeImplFromJson(json);

  @override
  @JsonKey(name: 'challenge_id')
  final String challengeId;
  @override
  @JsonKey(name: 'factor_type')
  final String factorType;
  @override
  @JsonKey(name: 'expires_at')
  final String expiresAt;

  @override
  String toString() {
    return 'SigninChallenge(challengeId: $challengeId, factorType: $factorType, expiresAt: $expiresAt)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SigninChallengeImpl &&
            (identical(other.challengeId, challengeId) ||
                other.challengeId == challengeId) &&
            (identical(other.factorType, factorType) ||
                other.factorType == factorType) &&
            (identical(other.expiresAt, expiresAt) ||
                other.expiresAt == expiresAt));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, challengeId, factorType, expiresAt);

  /// Create a copy of SigninChallenge
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SigninChallengeImplCopyWith<_$SigninChallengeImpl> get copyWith =>
      __$$SigninChallengeImplCopyWithImpl<_$SigninChallengeImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SigninChallengeImplToJson(
      this,
    );
  }
}

abstract class _SigninChallenge implements SigninChallenge {
  const factory _SigninChallenge(
          {@JsonKey(name: 'challenge_id') required final String challengeId,
          @JsonKey(name: 'factor_type') required final String factorType,
          @JsonKey(name: 'expires_at') required final String expiresAt}) =
      _$SigninChallengeImpl;

  factory _SigninChallenge.fromJson(Map<String, dynamic> json) =
      _$SigninChallengeImpl.fromJson;

  @override
  @JsonKey(name: 'challenge_id')
  String get challengeId;
  @override
  @JsonKey(name: 'factor_type')
  String get factorType;
  @override
  @JsonKey(name: 'expires_at')
  String get expiresAt;

  /// Create a copy of SigninChallenge
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SigninChallengeImplCopyWith<_$SigninChallengeImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
