// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'oidc_client.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

OidcClient _$OidcClientFromJson(Map<String, dynamic> json) {
  return _OidcClient.fromJson(json);
}

/// @nodoc
mixin _$OidcClient {
  @JsonKey(name: 'client_id')
  String get clientId => throw _privateConstructorUsedError;
  @JsonKey(name: 'application_id')
  String get applicationId => throw _privateConstructorUsedError;
  String get name => throw _privateConstructorUsedError;
  @JsonKey(name: 'client_type')
  String get clientType => throw _privateConstructorUsedError;
  @JsonKey(name: 'application_type')
  String get applicationType => throw _privateConstructorUsedError;
  @JsonKey(name: 'redirect_uris')
  List<String> get redirectUris => throw _privateConstructorUsedError;
  @JsonKey(name: 'created_at')
  String get createdAt => throw _privateConstructorUsedError;

  /// Serializes this OidcClient to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of OidcClient
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $OidcClientCopyWith<OidcClient> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $OidcClientCopyWith<$Res> {
  factory $OidcClientCopyWith(
          OidcClient value, $Res Function(OidcClient) then) =
      _$OidcClientCopyWithImpl<$Res, OidcClient>;
  @useResult
  $Res call(
      {@JsonKey(name: 'client_id') String clientId,
      @JsonKey(name: 'application_id') String applicationId,
      String name,
      @JsonKey(name: 'client_type') String clientType,
      @JsonKey(name: 'application_type') String applicationType,
      @JsonKey(name: 'redirect_uris') List<String> redirectUris,
      @JsonKey(name: 'created_at') String createdAt});
}

/// @nodoc
class _$OidcClientCopyWithImpl<$Res, $Val extends OidcClient>
    implements $OidcClientCopyWith<$Res> {
  _$OidcClientCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of OidcClient
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? clientId = null,
    Object? applicationId = null,
    Object? name = null,
    Object? clientType = null,
    Object? applicationType = null,
    Object? redirectUris = null,
    Object? createdAt = null,
  }) {
    return _then(_value.copyWith(
      clientId: null == clientId
          ? _value.clientId
          : clientId // ignore: cast_nullable_to_non_nullable
              as String,
      applicationId: null == applicationId
          ? _value.applicationId
          : applicationId // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      clientType: null == clientType
          ? _value.clientType
          : clientType // ignore: cast_nullable_to_non_nullable
              as String,
      applicationType: null == applicationType
          ? _value.applicationType
          : applicationType // ignore: cast_nullable_to_non_nullable
              as String,
      redirectUris: null == redirectUris
          ? _value.redirectUris
          : redirectUris // ignore: cast_nullable_to_non_nullable
              as List<String>,
      createdAt: null == createdAt
          ? _value.createdAt
          : createdAt // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$OidcClientImplCopyWith<$Res>
    implements $OidcClientCopyWith<$Res> {
  factory _$$OidcClientImplCopyWith(
          _$OidcClientImpl value, $Res Function(_$OidcClientImpl) then) =
      __$$OidcClientImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'client_id') String clientId,
      @JsonKey(name: 'application_id') String applicationId,
      String name,
      @JsonKey(name: 'client_type') String clientType,
      @JsonKey(name: 'application_type') String applicationType,
      @JsonKey(name: 'redirect_uris') List<String> redirectUris,
      @JsonKey(name: 'created_at') String createdAt});
}

/// @nodoc
class __$$OidcClientImplCopyWithImpl<$Res>
    extends _$OidcClientCopyWithImpl<$Res, _$OidcClientImpl>
    implements _$$OidcClientImplCopyWith<$Res> {
  __$$OidcClientImplCopyWithImpl(
      _$OidcClientImpl _value, $Res Function(_$OidcClientImpl) _then)
      : super(_value, _then);

  /// Create a copy of OidcClient
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? clientId = null,
    Object? applicationId = null,
    Object? name = null,
    Object? clientType = null,
    Object? applicationType = null,
    Object? redirectUris = null,
    Object? createdAt = null,
  }) {
    return _then(_$OidcClientImpl(
      clientId: null == clientId
          ? _value.clientId
          : clientId // ignore: cast_nullable_to_non_nullable
              as String,
      applicationId: null == applicationId
          ? _value.applicationId
          : applicationId // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      clientType: null == clientType
          ? _value.clientType
          : clientType // ignore: cast_nullable_to_non_nullable
              as String,
      applicationType: null == applicationType
          ? _value.applicationType
          : applicationType // ignore: cast_nullable_to_non_nullable
              as String,
      redirectUris: null == redirectUris
          ? _value._redirectUris
          : redirectUris // ignore: cast_nullable_to_non_nullable
              as List<String>,
      createdAt: null == createdAt
          ? _value.createdAt
          : createdAt // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$OidcClientImpl implements _OidcClient {
  const _$OidcClientImpl(
      {@JsonKey(name: 'client_id') required this.clientId,
      @JsonKey(name: 'application_id') required this.applicationId,
      required this.name,
      @JsonKey(name: 'client_type') required this.clientType,
      @JsonKey(name: 'application_type') required this.applicationType,
      @JsonKey(name: 'redirect_uris') required final List<String> redirectUris,
      @JsonKey(name: 'created_at') required this.createdAt})
      : _redirectUris = redirectUris;

  factory _$OidcClientImpl.fromJson(Map<String, dynamic> json) =>
      _$$OidcClientImplFromJson(json);

  @override
  @JsonKey(name: 'client_id')
  final String clientId;
  @override
  @JsonKey(name: 'application_id')
  final String applicationId;
  @override
  final String name;
  @override
  @JsonKey(name: 'client_type')
  final String clientType;
  @override
  @JsonKey(name: 'application_type')
  final String applicationType;
  final List<String> _redirectUris;
  @override
  @JsonKey(name: 'redirect_uris')
  List<String> get redirectUris {
    if (_redirectUris is EqualUnmodifiableListView) return _redirectUris;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_redirectUris);
  }

  @override
  @JsonKey(name: 'created_at')
  final String createdAt;

  @override
  String toString() {
    return 'OidcClient(clientId: $clientId, applicationId: $applicationId, name: $name, clientType: $clientType, applicationType: $applicationType, redirectUris: $redirectUris, createdAt: $createdAt)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$OidcClientImpl &&
            (identical(other.clientId, clientId) ||
                other.clientId == clientId) &&
            (identical(other.applicationId, applicationId) ||
                other.applicationId == applicationId) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.clientType, clientType) ||
                other.clientType == clientType) &&
            (identical(other.applicationType, applicationType) ||
                other.applicationType == applicationType) &&
            const DeepCollectionEquality()
                .equals(other._redirectUris, _redirectUris) &&
            (identical(other.createdAt, createdAt) ||
                other.createdAt == createdAt));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      clientId,
      applicationId,
      name,
      clientType,
      applicationType,
      const DeepCollectionEquality().hash(_redirectUris),
      createdAt);

  /// Create a copy of OidcClient
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$OidcClientImplCopyWith<_$OidcClientImpl> get copyWith =>
      __$$OidcClientImplCopyWithImpl<_$OidcClientImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$OidcClientImplToJson(
      this,
    );
  }
}

abstract class _OidcClient implements OidcClient {
  const factory _OidcClient(
      {@JsonKey(name: 'client_id') required final String clientId,
      @JsonKey(name: 'application_id') required final String applicationId,
      required final String name,
      @JsonKey(name: 'client_type') required final String clientType,
      @JsonKey(name: 'application_type') required final String applicationType,
      @JsonKey(name: 'redirect_uris') required final List<String> redirectUris,
      @JsonKey(name: 'created_at')
      required final String createdAt}) = _$OidcClientImpl;

  factory _OidcClient.fromJson(Map<String, dynamic> json) =
      _$OidcClientImpl.fromJson;

  @override
  @JsonKey(name: 'client_id')
  String get clientId;
  @override
  @JsonKey(name: 'application_id')
  String get applicationId;
  @override
  String get name;
  @override
  @JsonKey(name: 'client_type')
  String get clientType;
  @override
  @JsonKey(name: 'application_type')
  String get applicationType;
  @override
  @JsonKey(name: 'redirect_uris')
  List<String> get redirectUris;
  @override
  @JsonKey(name: 'created_at')
  String get createdAt;

  /// Create a copy of OidcClient
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$OidcClientImplCopyWith<_$OidcClientImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

CreateOidcClientRequest _$CreateOidcClientRequestFromJson(
    Map<String, dynamic> json) {
  return _CreateOidcClientRequest.fromJson(json);
}

/// @nodoc
mixin _$CreateOidcClientRequest {
  String get name => throw _privateConstructorUsedError;
  @JsonKey(name: 'redirect_uris')
  List<String> get redirectUris => throw _privateConstructorUsedError;

  /// Serializes this CreateOidcClientRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of CreateOidcClientRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $CreateOidcClientRequestCopyWith<CreateOidcClientRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $CreateOidcClientRequestCopyWith<$Res> {
  factory $CreateOidcClientRequestCopyWith(CreateOidcClientRequest value,
          $Res Function(CreateOidcClientRequest) then) =
      _$CreateOidcClientRequestCopyWithImpl<$Res, CreateOidcClientRequest>;
  @useResult
  $Res call(
      {String name, @JsonKey(name: 'redirect_uris') List<String> redirectUris});
}

/// @nodoc
class _$CreateOidcClientRequestCopyWithImpl<$Res,
        $Val extends CreateOidcClientRequest>
    implements $CreateOidcClientRequestCopyWith<$Res> {
  _$CreateOidcClientRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of CreateOidcClientRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = null,
    Object? redirectUris = null,
  }) {
    return _then(_value.copyWith(
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      redirectUris: null == redirectUris
          ? _value.redirectUris
          : redirectUris // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$CreateOidcClientRequestImplCopyWith<$Res>
    implements $CreateOidcClientRequestCopyWith<$Res> {
  factory _$$CreateOidcClientRequestImplCopyWith(
          _$CreateOidcClientRequestImpl value,
          $Res Function(_$CreateOidcClientRequestImpl) then) =
      __$$CreateOidcClientRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String name, @JsonKey(name: 'redirect_uris') List<String> redirectUris});
}

/// @nodoc
class __$$CreateOidcClientRequestImplCopyWithImpl<$Res>
    extends _$CreateOidcClientRequestCopyWithImpl<$Res,
        _$CreateOidcClientRequestImpl>
    implements _$$CreateOidcClientRequestImplCopyWith<$Res> {
  __$$CreateOidcClientRequestImplCopyWithImpl(
      _$CreateOidcClientRequestImpl _value,
      $Res Function(_$CreateOidcClientRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of CreateOidcClientRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = null,
    Object? redirectUris = null,
  }) {
    return _then(_$CreateOidcClientRequestImpl(
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      redirectUris: null == redirectUris
          ? _value._redirectUris
          : redirectUris // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$CreateOidcClientRequestImpl implements _CreateOidcClientRequest {
  const _$CreateOidcClientRequestImpl(
      {required this.name,
      @JsonKey(name: 'redirect_uris') required final List<String> redirectUris})
      : _redirectUris = redirectUris;

  factory _$CreateOidcClientRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$CreateOidcClientRequestImplFromJson(json);

  @override
  final String name;
  final List<String> _redirectUris;
  @override
  @JsonKey(name: 'redirect_uris')
  List<String> get redirectUris {
    if (_redirectUris is EqualUnmodifiableListView) return _redirectUris;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_redirectUris);
  }

  @override
  String toString() {
    return 'CreateOidcClientRequest(name: $name, redirectUris: $redirectUris)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CreateOidcClientRequestImpl &&
            (identical(other.name, name) || other.name == name) &&
            const DeepCollectionEquality()
                .equals(other._redirectUris, _redirectUris));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType, name, const DeepCollectionEquality().hash(_redirectUris));

  /// Create a copy of CreateOidcClientRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CreateOidcClientRequestImplCopyWith<_$CreateOidcClientRequestImpl>
      get copyWith => __$$CreateOidcClientRequestImplCopyWithImpl<
          _$CreateOidcClientRequestImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$CreateOidcClientRequestImplToJson(
      this,
    );
  }
}

abstract class _CreateOidcClientRequest implements CreateOidcClientRequest {
  const factory _CreateOidcClientRequest(
          {required final String name,
          @JsonKey(name: 'redirect_uris')
          required final List<String> redirectUris}) =
      _$CreateOidcClientRequestImpl;

  factory _CreateOidcClientRequest.fromJson(Map<String, dynamic> json) =
      _$CreateOidcClientRequestImpl.fromJson;

  @override
  String get name;
  @override
  @JsonKey(name: 'redirect_uris')
  List<String> get redirectUris;

  /// Create a copy of CreateOidcClientRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CreateOidcClientRequestImplCopyWith<_$CreateOidcClientRequestImpl>
      get copyWith => throw _privateConstructorUsedError;
}

PatchOidcClientRequest _$PatchOidcClientRequestFromJson(
    Map<String, dynamic> json) {
  return _PatchOidcClientRequest.fromJson(json);
}

/// @nodoc
mixin _$PatchOidcClientRequest {
  String? get name => throw _privateConstructorUsedError;
  @JsonKey(name: 'redirect_uris')
  List<String>? get redirectUris => throw _privateConstructorUsedError;

  /// Serializes this PatchOidcClientRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of PatchOidcClientRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $PatchOidcClientRequestCopyWith<PatchOidcClientRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PatchOidcClientRequestCopyWith<$Res> {
  factory $PatchOidcClientRequestCopyWith(PatchOidcClientRequest value,
          $Res Function(PatchOidcClientRequest) then) =
      _$PatchOidcClientRequestCopyWithImpl<$Res, PatchOidcClientRequest>;
  @useResult
  $Res call(
      {String? name,
      @JsonKey(name: 'redirect_uris') List<String>? redirectUris});
}

/// @nodoc
class _$PatchOidcClientRequestCopyWithImpl<$Res,
        $Val extends PatchOidcClientRequest>
    implements $PatchOidcClientRequestCopyWith<$Res> {
  _$PatchOidcClientRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PatchOidcClientRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = freezed,
    Object? redirectUris = freezed,
  }) {
    return _then(_value.copyWith(
      name: freezed == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String?,
      redirectUris: freezed == redirectUris
          ? _value.redirectUris
          : redirectUris // ignore: cast_nullable_to_non_nullable
              as List<String>?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$PatchOidcClientRequestImplCopyWith<$Res>
    implements $PatchOidcClientRequestCopyWith<$Res> {
  factory _$$PatchOidcClientRequestImplCopyWith(
          _$PatchOidcClientRequestImpl value,
          $Res Function(_$PatchOidcClientRequestImpl) then) =
      __$$PatchOidcClientRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String? name,
      @JsonKey(name: 'redirect_uris') List<String>? redirectUris});
}

/// @nodoc
class __$$PatchOidcClientRequestImplCopyWithImpl<$Res>
    extends _$PatchOidcClientRequestCopyWithImpl<$Res,
        _$PatchOidcClientRequestImpl>
    implements _$$PatchOidcClientRequestImplCopyWith<$Res> {
  __$$PatchOidcClientRequestImplCopyWithImpl(
      _$PatchOidcClientRequestImpl _value,
      $Res Function(_$PatchOidcClientRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of PatchOidcClientRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = freezed,
    Object? redirectUris = freezed,
  }) {
    return _then(_$PatchOidcClientRequestImpl(
      name: freezed == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String?,
      redirectUris: freezed == redirectUris
          ? _value._redirectUris
          : redirectUris // ignore: cast_nullable_to_non_nullable
              as List<String>?,
    ));
  }
}

/// @nodoc

@JsonSerializable(includeIfNull: false)
class _$PatchOidcClientRequestImpl implements _PatchOidcClientRequest {
  const _$PatchOidcClientRequestImpl(
      {this.name,
      @JsonKey(name: 'redirect_uris') final List<String>? redirectUris})
      : _redirectUris = redirectUris;

  factory _$PatchOidcClientRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$PatchOidcClientRequestImplFromJson(json);

  @override
  final String? name;
  final List<String>? _redirectUris;
  @override
  @JsonKey(name: 'redirect_uris')
  List<String>? get redirectUris {
    final value = _redirectUris;
    if (value == null) return null;
    if (_redirectUris is EqualUnmodifiableListView) return _redirectUris;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(value);
  }

  @override
  String toString() {
    return 'PatchOidcClientRequest(name: $name, redirectUris: $redirectUris)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PatchOidcClientRequestImpl &&
            (identical(other.name, name) || other.name == name) &&
            const DeepCollectionEquality()
                .equals(other._redirectUris, _redirectUris));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType, name, const DeepCollectionEquality().hash(_redirectUris));

  /// Create a copy of PatchOidcClientRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PatchOidcClientRequestImplCopyWith<_$PatchOidcClientRequestImpl>
      get copyWith => __$$PatchOidcClientRequestImplCopyWithImpl<
          _$PatchOidcClientRequestImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$PatchOidcClientRequestImplToJson(
      this,
    );
  }
}

abstract class _PatchOidcClientRequest implements PatchOidcClientRequest {
  const factory _PatchOidcClientRequest(
          {final String? name,
          @JsonKey(name: 'redirect_uris') final List<String>? redirectUris}) =
      _$PatchOidcClientRequestImpl;

  factory _PatchOidcClientRequest.fromJson(Map<String, dynamic> json) =
      _$PatchOidcClientRequestImpl.fromJson;

  @override
  String? get name;
  @override
  @JsonKey(name: 'redirect_uris')
  List<String>? get redirectUris;

  /// Create a copy of PatchOidcClientRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PatchOidcClientRequestImplCopyWith<_$PatchOidcClientRequestImpl>
      get copyWith => throw _privateConstructorUsedError;
}
