// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'secret.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

Secret _$SecretFromJson(Map<String, dynamic> json) {
  return _Secret.fromJson(json);
}

/// @nodoc
mixin _$Secret {
  String get id => throw _privateConstructorUsedError;
  String? get secret => throw _privateConstructorUsedError;
  @JsonKey(name: 'created_at')
  String get createdAt => throw _privateConstructorUsedError;
  @JsonKey(name: 'revoked_at')
  String? get revokedAt => throw _privateConstructorUsedError;
  @JsonKey(name: 'application_ids')
  List<String> get applicationIds => throw _privateConstructorUsedError;

  /// Serializes this Secret to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of Secret
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $SecretCopyWith<Secret> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SecretCopyWith<$Res> {
  factory $SecretCopyWith(Secret value, $Res Function(Secret) then) =
      _$SecretCopyWithImpl<$Res, Secret>;
  @useResult
  $Res call(
      {String id,
      String? secret,
      @JsonKey(name: 'created_at') String createdAt,
      @JsonKey(name: 'revoked_at') String? revokedAt,
      @JsonKey(name: 'application_ids') List<String> applicationIds});
}

/// @nodoc
class _$SecretCopyWithImpl<$Res, $Val extends Secret>
    implements $SecretCopyWith<$Res> {
  _$SecretCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Secret
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? secret = freezed,
    Object? createdAt = null,
    Object? revokedAt = freezed,
    Object? applicationIds = null,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      secret: freezed == secret
          ? _value.secret
          : secret // ignore: cast_nullable_to_non_nullable
              as String?,
      createdAt: null == createdAt
          ? _value.createdAt
          : createdAt // ignore: cast_nullable_to_non_nullable
              as String,
      revokedAt: freezed == revokedAt
          ? _value.revokedAt
          : revokedAt // ignore: cast_nullable_to_non_nullable
              as String?,
      applicationIds: null == applicationIds
          ? _value.applicationIds
          : applicationIds // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SecretImplCopyWith<$Res> implements $SecretCopyWith<$Res> {
  factory _$$SecretImplCopyWith(
          _$SecretImpl value, $Res Function(_$SecretImpl) then) =
      __$$SecretImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String id,
      String? secret,
      @JsonKey(name: 'created_at') String createdAt,
      @JsonKey(name: 'revoked_at') String? revokedAt,
      @JsonKey(name: 'application_ids') List<String> applicationIds});
}

/// @nodoc
class __$$SecretImplCopyWithImpl<$Res>
    extends _$SecretCopyWithImpl<$Res, _$SecretImpl>
    implements _$$SecretImplCopyWith<$Res> {
  __$$SecretImplCopyWithImpl(
      _$SecretImpl _value, $Res Function(_$SecretImpl) _then)
      : super(_value, _then);

  /// Create a copy of Secret
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? secret = freezed,
    Object? createdAt = null,
    Object? revokedAt = freezed,
    Object? applicationIds = null,
  }) {
    return _then(_$SecretImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      secret: freezed == secret
          ? _value.secret
          : secret // ignore: cast_nullable_to_non_nullable
              as String?,
      createdAt: null == createdAt
          ? _value.createdAt
          : createdAt // ignore: cast_nullable_to_non_nullable
              as String,
      revokedAt: freezed == revokedAt
          ? _value.revokedAt
          : revokedAt // ignore: cast_nullable_to_non_nullable
              as String?,
      applicationIds: null == applicationIds
          ? _value._applicationIds
          : applicationIds // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SecretImpl implements _Secret {
  const _$SecretImpl(
      {required this.id,
      this.secret,
      @JsonKey(name: 'created_at') required this.createdAt,
      @JsonKey(name: 'revoked_at') this.revokedAt,
      @JsonKey(name: 'application_ids')
      required final List<String> applicationIds})
      : _applicationIds = applicationIds;

  factory _$SecretImpl.fromJson(Map<String, dynamic> json) =>
      _$$SecretImplFromJson(json);

  @override
  final String id;
  @override
  final String? secret;
  @override
  @JsonKey(name: 'created_at')
  final String createdAt;
  @override
  @JsonKey(name: 'revoked_at')
  final String? revokedAt;
  final List<String> _applicationIds;
  @override
  @JsonKey(name: 'application_ids')
  List<String> get applicationIds {
    if (_applicationIds is EqualUnmodifiableListView) return _applicationIds;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_applicationIds);
  }

  @override
  String toString() {
    return 'Secret(id: $id, secret: $secret, createdAt: $createdAt, revokedAt: $revokedAt, applicationIds: $applicationIds)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SecretImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.secret, secret) || other.secret == secret) &&
            (identical(other.createdAt, createdAt) ||
                other.createdAt == createdAt) &&
            (identical(other.revokedAt, revokedAt) ||
                other.revokedAt == revokedAt) &&
            const DeepCollectionEquality()
                .equals(other._applicationIds, _applicationIds));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, secret, createdAt, revokedAt,
      const DeepCollectionEquality().hash(_applicationIds));

  /// Create a copy of Secret
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SecretImplCopyWith<_$SecretImpl> get copyWith =>
      __$$SecretImplCopyWithImpl<_$SecretImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SecretImplToJson(
      this,
    );
  }
}

abstract class _Secret implements Secret {
  const factory _Secret(
      {required final String id,
      final String? secret,
      @JsonKey(name: 'created_at') required final String createdAt,
      @JsonKey(name: 'revoked_at') final String? revokedAt,
      @JsonKey(name: 'application_ids')
      required final List<String> applicationIds}) = _$SecretImpl;

  factory _Secret.fromJson(Map<String, dynamic> json) = _$SecretImpl.fromJson;

  @override
  String get id;
  @override
  String? get secret;
  @override
  @JsonKey(name: 'created_at')
  String get createdAt;
  @override
  @JsonKey(name: 'revoked_at')
  String? get revokedAt;
  @override
  @JsonKey(name: 'application_ids')
  List<String> get applicationIds;

  /// Create a copy of Secret
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SecretImplCopyWith<_$SecretImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

CreateSecretResponse _$CreateSecretResponseFromJson(Map<String, dynamic> json) {
  return _CreateSecretResponse.fromJson(json);
}

/// @nodoc
mixin _$CreateSecretResponse {
  String get id => throw _privateConstructorUsedError;
  String get secret => throw _privateConstructorUsedError;

  /// Serializes this CreateSecretResponse to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of CreateSecretResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $CreateSecretResponseCopyWith<CreateSecretResponse> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $CreateSecretResponseCopyWith<$Res> {
  factory $CreateSecretResponseCopyWith(CreateSecretResponse value,
          $Res Function(CreateSecretResponse) then) =
      _$CreateSecretResponseCopyWithImpl<$Res, CreateSecretResponse>;
  @useResult
  $Res call({String id, String secret});
}

/// @nodoc
class _$CreateSecretResponseCopyWithImpl<$Res,
        $Val extends CreateSecretResponse>
    implements $CreateSecretResponseCopyWith<$Res> {
  _$CreateSecretResponseCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of CreateSecretResponse
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? secret = null,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      secret: null == secret
          ? _value.secret
          : secret // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$CreateSecretResponseImplCopyWith<$Res>
    implements $CreateSecretResponseCopyWith<$Res> {
  factory _$$CreateSecretResponseImplCopyWith(_$CreateSecretResponseImpl value,
          $Res Function(_$CreateSecretResponseImpl) then) =
      __$$CreateSecretResponseImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String id, String secret});
}

/// @nodoc
class __$$CreateSecretResponseImplCopyWithImpl<$Res>
    extends _$CreateSecretResponseCopyWithImpl<$Res, _$CreateSecretResponseImpl>
    implements _$$CreateSecretResponseImplCopyWith<$Res> {
  __$$CreateSecretResponseImplCopyWithImpl(_$CreateSecretResponseImpl _value,
      $Res Function(_$CreateSecretResponseImpl) _then)
      : super(_value, _then);

  /// Create a copy of CreateSecretResponse
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? secret = null,
  }) {
    return _then(_$CreateSecretResponseImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      secret: null == secret
          ? _value.secret
          : secret // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$CreateSecretResponseImpl implements _CreateSecretResponse {
  const _$CreateSecretResponseImpl({required this.id, required this.secret});

  factory _$CreateSecretResponseImpl.fromJson(Map<String, dynamic> json) =>
      _$$CreateSecretResponseImplFromJson(json);

  @override
  final String id;
  @override
  final String secret;

  @override
  String toString() {
    return 'CreateSecretResponse(id: $id, secret: $secret)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CreateSecretResponseImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.secret, secret) || other.secret == secret));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, secret);

  /// Create a copy of CreateSecretResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CreateSecretResponseImplCopyWith<_$CreateSecretResponseImpl>
      get copyWith =>
          __$$CreateSecretResponseImplCopyWithImpl<_$CreateSecretResponseImpl>(
              this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$CreateSecretResponseImplToJson(
      this,
    );
  }
}

abstract class _CreateSecretResponse implements CreateSecretResponse {
  const factory _CreateSecretResponse(
      {required final String id,
      required final String secret}) = _$CreateSecretResponseImpl;

  factory _CreateSecretResponse.fromJson(Map<String, dynamic> json) =
      _$CreateSecretResponseImpl.fromJson;

  @override
  String get id;
  @override
  String get secret;

  /// Create a copy of CreateSecretResponse
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CreateSecretResponseImplCopyWith<_$CreateSecretResponseImpl>
      get copyWith => throw _privateConstructorUsedError;
}

BindSecretRequest _$BindSecretRequestFromJson(Map<String, dynamic> json) {
  return _BindSecretRequest.fromJson(json);
}

/// @nodoc
mixin _$BindSecretRequest {
  @JsonKey(name: 'application_id')
  String get applicationId => throw _privateConstructorUsedError;

  /// Serializes this BindSecretRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of BindSecretRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $BindSecretRequestCopyWith<BindSecretRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $BindSecretRequestCopyWith<$Res> {
  factory $BindSecretRequestCopyWith(
          BindSecretRequest value, $Res Function(BindSecretRequest) then) =
      _$BindSecretRequestCopyWithImpl<$Res, BindSecretRequest>;
  @useResult
  $Res call({@JsonKey(name: 'application_id') String applicationId});
}

/// @nodoc
class _$BindSecretRequestCopyWithImpl<$Res, $Val extends BindSecretRequest>
    implements $BindSecretRequestCopyWith<$Res> {
  _$BindSecretRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of BindSecretRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? applicationId = null,
  }) {
    return _then(_value.copyWith(
      applicationId: null == applicationId
          ? _value.applicationId
          : applicationId // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$BindSecretRequestImplCopyWith<$Res>
    implements $BindSecretRequestCopyWith<$Res> {
  factory _$$BindSecretRequestImplCopyWith(_$BindSecretRequestImpl value,
          $Res Function(_$BindSecretRequestImpl) then) =
      __$$BindSecretRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({@JsonKey(name: 'application_id') String applicationId});
}

/// @nodoc
class __$$BindSecretRequestImplCopyWithImpl<$Res>
    extends _$BindSecretRequestCopyWithImpl<$Res, _$BindSecretRequestImpl>
    implements _$$BindSecretRequestImplCopyWith<$Res> {
  __$$BindSecretRequestImplCopyWithImpl(_$BindSecretRequestImpl _value,
      $Res Function(_$BindSecretRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of BindSecretRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? applicationId = null,
  }) {
    return _then(_$BindSecretRequestImpl(
      applicationId: null == applicationId
          ? _value.applicationId
          : applicationId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$BindSecretRequestImpl implements _BindSecretRequest {
  const _$BindSecretRequestImpl(
      {@JsonKey(name: 'application_id') required this.applicationId});

  factory _$BindSecretRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$BindSecretRequestImplFromJson(json);

  @override
  @JsonKey(name: 'application_id')
  final String applicationId;

  @override
  String toString() {
    return 'BindSecretRequest(applicationId: $applicationId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$BindSecretRequestImpl &&
            (identical(other.applicationId, applicationId) ||
                other.applicationId == applicationId));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, applicationId);

  /// Create a copy of BindSecretRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$BindSecretRequestImplCopyWith<_$BindSecretRequestImpl> get copyWith =>
      __$$BindSecretRequestImplCopyWithImpl<_$BindSecretRequestImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$BindSecretRequestImplToJson(
      this,
    );
  }
}

abstract class _BindSecretRequest implements BindSecretRequest {
  const factory _BindSecretRequest(
      {@JsonKey(name: 'application_id')
      required final String applicationId}) = _$BindSecretRequestImpl;

  factory _BindSecretRequest.fromJson(Map<String, dynamic> json) =
      _$BindSecretRequestImpl.fromJson;

  @override
  @JsonKey(name: 'application_id')
  String get applicationId;

  /// Create a copy of BindSecretRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$BindSecretRequestImplCopyWith<_$BindSecretRequestImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
