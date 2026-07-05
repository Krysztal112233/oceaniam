// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'administrator.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

Administrator _$AdministratorFromJson(Map<String, dynamic> json) {
  return _Administrator.fromJson(json);
}

/// @nodoc
mixin _$Administrator {
  String get id => throw _privateConstructorUsedError;
  String get name => throw _privateConstructorUsedError;
  String? get role => throw _privateConstructorUsedError;

  /// Serializes this Administrator to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of Administrator
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $AdministratorCopyWith<Administrator> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AdministratorCopyWith<$Res> {
  factory $AdministratorCopyWith(
          Administrator value, $Res Function(Administrator) then) =
      _$AdministratorCopyWithImpl<$Res, Administrator>;
  @useResult
  $Res call({String id, String name, String? role});
}

/// @nodoc
class _$AdministratorCopyWithImpl<$Res, $Val extends Administrator>
    implements $AdministratorCopyWith<$Res> {
  _$AdministratorCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Administrator
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? role = freezed,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      role: freezed == role
          ? _value.role
          : role // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$AdministratorImplCopyWith<$Res>
    implements $AdministratorCopyWith<$Res> {
  factory _$$AdministratorImplCopyWith(
          _$AdministratorImpl value, $Res Function(_$AdministratorImpl) then) =
      __$$AdministratorImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String id, String name, String? role});
}

/// @nodoc
class __$$AdministratorImplCopyWithImpl<$Res>
    extends _$AdministratorCopyWithImpl<$Res, _$AdministratorImpl>
    implements _$$AdministratorImplCopyWith<$Res> {
  __$$AdministratorImplCopyWithImpl(
      _$AdministratorImpl _value, $Res Function(_$AdministratorImpl) _then)
      : super(_value, _then);

  /// Create a copy of Administrator
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? role = freezed,
  }) {
    return _then(_$AdministratorImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      role: freezed == role
          ? _value.role
          : role // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$AdministratorImpl implements _Administrator {
  const _$AdministratorImpl({required this.id, required this.name, this.role});

  factory _$AdministratorImpl.fromJson(Map<String, dynamic> json) =>
      _$$AdministratorImplFromJson(json);

  @override
  final String id;
  @override
  final String name;
  @override
  final String? role;

  @override
  String toString() {
    return 'Administrator(id: $id, name: $name, role: $role)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AdministratorImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.role, role) || other.role == role));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, name, role);

  /// Create a copy of Administrator
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AdministratorImplCopyWith<_$AdministratorImpl> get copyWith =>
      __$$AdministratorImplCopyWithImpl<_$AdministratorImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$AdministratorImplToJson(
      this,
    );
  }
}

abstract class _Administrator implements Administrator {
  const factory _Administrator(
      {required final String id,
      required final String name,
      final String? role}) = _$AdministratorImpl;

  factory _Administrator.fromJson(Map<String, dynamic> json) =
      _$AdministratorImpl.fromJson;

  @override
  String get id;
  @override
  String get name;
  @override
  String? get role;

  /// Create a copy of Administrator
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AdministratorImplCopyWith<_$AdministratorImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

AdministratorProfile _$AdministratorProfileFromJson(Map<String, dynamic> json) {
  return _AdministratorProfile.fromJson(json);
}

/// @nodoc
mixin _$AdministratorProfile {
  String get id => throw _privateConstructorUsedError;
  String get name => throw _privateConstructorUsedError;
  String? get role => throw _privateConstructorUsedError;
  List<String> get permissions => throw _privateConstructorUsedError;

  /// Serializes this AdministratorProfile to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of AdministratorProfile
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $AdministratorProfileCopyWith<AdministratorProfile> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AdministratorProfileCopyWith<$Res> {
  factory $AdministratorProfileCopyWith(AdministratorProfile value,
          $Res Function(AdministratorProfile) then) =
      _$AdministratorProfileCopyWithImpl<$Res, AdministratorProfile>;
  @useResult
  $Res call({String id, String name, String? role, List<String> permissions});
}

/// @nodoc
class _$AdministratorProfileCopyWithImpl<$Res,
        $Val extends AdministratorProfile>
    implements $AdministratorProfileCopyWith<$Res> {
  _$AdministratorProfileCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of AdministratorProfile
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? role = freezed,
    Object? permissions = null,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      role: freezed == role
          ? _value.role
          : role // ignore: cast_nullable_to_non_nullable
              as String?,
      permissions: null == permissions
          ? _value.permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$AdministratorProfileImplCopyWith<$Res>
    implements $AdministratorProfileCopyWith<$Res> {
  factory _$$AdministratorProfileImplCopyWith(_$AdministratorProfileImpl value,
          $Res Function(_$AdministratorProfileImpl) then) =
      __$$AdministratorProfileImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String id, String name, String? role, List<String> permissions});
}

/// @nodoc
class __$$AdministratorProfileImplCopyWithImpl<$Res>
    extends _$AdministratorProfileCopyWithImpl<$Res, _$AdministratorProfileImpl>
    implements _$$AdministratorProfileImplCopyWith<$Res> {
  __$$AdministratorProfileImplCopyWithImpl(_$AdministratorProfileImpl _value,
      $Res Function(_$AdministratorProfileImpl) _then)
      : super(_value, _then);

  /// Create a copy of AdministratorProfile
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? role = freezed,
    Object? permissions = null,
  }) {
    return _then(_$AdministratorProfileImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      role: freezed == role
          ? _value.role
          : role // ignore: cast_nullable_to_non_nullable
              as String?,
      permissions: null == permissions
          ? _value._permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$AdministratorProfileImpl implements _AdministratorProfile {
  const _$AdministratorProfileImpl(
      {required this.id,
      required this.name,
      this.role,
      required final List<String> permissions})
      : _permissions = permissions;

  factory _$AdministratorProfileImpl.fromJson(Map<String, dynamic> json) =>
      _$$AdministratorProfileImplFromJson(json);

  @override
  final String id;
  @override
  final String name;
  @override
  final String? role;
  final List<String> _permissions;
  @override
  List<String> get permissions {
    if (_permissions is EqualUnmodifiableListView) return _permissions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_permissions);
  }

  @override
  String toString() {
    return 'AdministratorProfile(id: $id, name: $name, role: $role, permissions: $permissions)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AdministratorProfileImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.role, role) || other.role == role) &&
            const DeepCollectionEquality()
                .equals(other._permissions, _permissions));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, name, role,
      const DeepCollectionEquality().hash(_permissions));

  /// Create a copy of AdministratorProfile
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AdministratorProfileImplCopyWith<_$AdministratorProfileImpl>
      get copyWith =>
          __$$AdministratorProfileImplCopyWithImpl<_$AdministratorProfileImpl>(
              this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$AdministratorProfileImplToJson(
      this,
    );
  }
}

abstract class _AdministratorProfile implements AdministratorProfile {
  const factory _AdministratorProfile(
      {required final String id,
      required final String name,
      final String? role,
      required final List<String> permissions}) = _$AdministratorProfileImpl;

  factory _AdministratorProfile.fromJson(Map<String, dynamic> json) =
      _$AdministratorProfileImpl.fromJson;

  @override
  String get id;
  @override
  String get name;
  @override
  String? get role;
  @override
  List<String> get permissions;

  /// Create a copy of AdministratorProfile
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AdministratorProfileImplCopyWith<_$AdministratorProfileImpl>
      get copyWith => throw _privateConstructorUsedError;
}

CreateAdministratorRequest _$CreateAdministratorRequestFromJson(
    Map<String, dynamic> json) {
  return _CreateAdministratorRequest.fromJson(json);
}

/// @nodoc
mixin _$CreateAdministratorRequest {
  String get name => throw _privateConstructorUsedError;
  String get password => throw _privateConstructorUsedError;

  /// Serializes this CreateAdministratorRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of CreateAdministratorRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $CreateAdministratorRequestCopyWith<CreateAdministratorRequest>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $CreateAdministratorRequestCopyWith<$Res> {
  factory $CreateAdministratorRequestCopyWith(CreateAdministratorRequest value,
          $Res Function(CreateAdministratorRequest) then) =
      _$CreateAdministratorRequestCopyWithImpl<$Res,
          CreateAdministratorRequest>;
  @useResult
  $Res call({String name, String password});
}

/// @nodoc
class _$CreateAdministratorRequestCopyWithImpl<$Res,
        $Val extends CreateAdministratorRequest>
    implements $CreateAdministratorRequestCopyWith<$Res> {
  _$CreateAdministratorRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of CreateAdministratorRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = null,
    Object? password = null,
  }) {
    return _then(_value.copyWith(
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      password: null == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$CreateAdministratorRequestImplCopyWith<$Res>
    implements $CreateAdministratorRequestCopyWith<$Res> {
  factory _$$CreateAdministratorRequestImplCopyWith(
          _$CreateAdministratorRequestImpl value,
          $Res Function(_$CreateAdministratorRequestImpl) then) =
      __$$CreateAdministratorRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String name, String password});
}

/// @nodoc
class __$$CreateAdministratorRequestImplCopyWithImpl<$Res>
    extends _$CreateAdministratorRequestCopyWithImpl<$Res,
        _$CreateAdministratorRequestImpl>
    implements _$$CreateAdministratorRequestImplCopyWith<$Res> {
  __$$CreateAdministratorRequestImplCopyWithImpl(
      _$CreateAdministratorRequestImpl _value,
      $Res Function(_$CreateAdministratorRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of CreateAdministratorRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = null,
    Object? password = null,
  }) {
    return _then(_$CreateAdministratorRequestImpl(
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      password: null == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$CreateAdministratorRequestImpl implements _CreateAdministratorRequest {
  const _$CreateAdministratorRequestImpl(
      {required this.name, required this.password});

  factory _$CreateAdministratorRequestImpl.fromJson(
          Map<String, dynamic> json) =>
      _$$CreateAdministratorRequestImplFromJson(json);

  @override
  final String name;
  @override
  final String password;

  @override
  String toString() {
    return 'CreateAdministratorRequest(name: $name, password: $password)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CreateAdministratorRequestImpl &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.password, password) ||
                other.password == password));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, name, password);

  /// Create a copy of CreateAdministratorRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CreateAdministratorRequestImplCopyWith<_$CreateAdministratorRequestImpl>
      get copyWith => __$$CreateAdministratorRequestImplCopyWithImpl<
          _$CreateAdministratorRequestImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$CreateAdministratorRequestImplToJson(
      this,
    );
  }
}

abstract class _CreateAdministratorRequest
    implements CreateAdministratorRequest {
  const factory _CreateAdministratorRequest(
      {required final String name,
      required final String password}) = _$CreateAdministratorRequestImpl;

  factory _CreateAdministratorRequest.fromJson(Map<String, dynamic> json) =
      _$CreateAdministratorRequestImpl.fromJson;

  @override
  String get name;
  @override
  String get password;

  /// Create a copy of CreateAdministratorRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CreateAdministratorRequestImplCopyWith<_$CreateAdministratorRequestImpl>
      get copyWith => throw _privateConstructorUsedError;
}

CreateAdministratorResponse _$CreateAdministratorResponseFromJson(
    Map<String, dynamic> json) {
  return _CreateAdministratorResponse.fromJson(json);
}

/// @nodoc
mixin _$CreateAdministratorResponse {
  String get id => throw _privateConstructorUsedError;
  String get name => throw _privateConstructorUsedError;
  String get password => throw _privateConstructorUsedError;

  /// Serializes this CreateAdministratorResponse to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of CreateAdministratorResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $CreateAdministratorResponseCopyWith<CreateAdministratorResponse>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $CreateAdministratorResponseCopyWith<$Res> {
  factory $CreateAdministratorResponseCopyWith(
          CreateAdministratorResponse value,
          $Res Function(CreateAdministratorResponse) then) =
      _$CreateAdministratorResponseCopyWithImpl<$Res,
          CreateAdministratorResponse>;
  @useResult
  $Res call({String id, String name, String password});
}

/// @nodoc
class _$CreateAdministratorResponseCopyWithImpl<$Res,
        $Val extends CreateAdministratorResponse>
    implements $CreateAdministratorResponseCopyWith<$Res> {
  _$CreateAdministratorResponseCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of CreateAdministratorResponse
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? password = null,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      password: null == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$CreateAdministratorResponseImplCopyWith<$Res>
    implements $CreateAdministratorResponseCopyWith<$Res> {
  factory _$$CreateAdministratorResponseImplCopyWith(
          _$CreateAdministratorResponseImpl value,
          $Res Function(_$CreateAdministratorResponseImpl) then) =
      __$$CreateAdministratorResponseImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String id, String name, String password});
}

/// @nodoc
class __$$CreateAdministratorResponseImplCopyWithImpl<$Res>
    extends _$CreateAdministratorResponseCopyWithImpl<$Res,
        _$CreateAdministratorResponseImpl>
    implements _$$CreateAdministratorResponseImplCopyWith<$Res> {
  __$$CreateAdministratorResponseImplCopyWithImpl(
      _$CreateAdministratorResponseImpl _value,
      $Res Function(_$CreateAdministratorResponseImpl) _then)
      : super(_value, _then);

  /// Create a copy of CreateAdministratorResponse
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? password = null,
  }) {
    return _then(_$CreateAdministratorResponseImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      password: null == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$CreateAdministratorResponseImpl
    implements _CreateAdministratorResponse {
  const _$CreateAdministratorResponseImpl(
      {required this.id, required this.name, required this.password});

  factory _$CreateAdministratorResponseImpl.fromJson(
          Map<String, dynamic> json) =>
      _$$CreateAdministratorResponseImplFromJson(json);

  @override
  final String id;
  @override
  final String name;
  @override
  final String password;

  @override
  String toString() {
    return 'CreateAdministratorResponse(id: $id, name: $name, password: $password)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CreateAdministratorResponseImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.password, password) ||
                other.password == password));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, name, password);

  /// Create a copy of CreateAdministratorResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CreateAdministratorResponseImplCopyWith<_$CreateAdministratorResponseImpl>
      get copyWith => __$$CreateAdministratorResponseImplCopyWithImpl<
          _$CreateAdministratorResponseImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$CreateAdministratorResponseImplToJson(
      this,
    );
  }
}

abstract class _CreateAdministratorResponse
    implements CreateAdministratorResponse {
  const factory _CreateAdministratorResponse(
      {required final String id,
      required final String name,
      required final String password}) = _$CreateAdministratorResponseImpl;

  factory _CreateAdministratorResponse.fromJson(Map<String, dynamic> json) =
      _$CreateAdministratorResponseImpl.fromJson;

  @override
  String get id;
  @override
  String get name;
  @override
  String get password;

  /// Create a copy of CreateAdministratorResponse
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CreateAdministratorResponseImplCopyWith<_$CreateAdministratorResponseImpl>
      get copyWith => throw _privateConstructorUsedError;
}

UpdateAdministratorRequest _$UpdateAdministratorRequestFromJson(
    Map<String, dynamic> json) {
  return _UpdateAdministratorRequest.fromJson(json);
}

/// @nodoc
mixin _$UpdateAdministratorRequest {
  String? get name => throw _privateConstructorUsedError;
  String? get password => throw _privateConstructorUsedError;

  /// Serializes this UpdateAdministratorRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of UpdateAdministratorRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $UpdateAdministratorRequestCopyWith<UpdateAdministratorRequest>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $UpdateAdministratorRequestCopyWith<$Res> {
  factory $UpdateAdministratorRequestCopyWith(UpdateAdministratorRequest value,
          $Res Function(UpdateAdministratorRequest) then) =
      _$UpdateAdministratorRequestCopyWithImpl<$Res,
          UpdateAdministratorRequest>;
  @useResult
  $Res call({String? name, String? password});
}

/// @nodoc
class _$UpdateAdministratorRequestCopyWithImpl<$Res,
        $Val extends UpdateAdministratorRequest>
    implements $UpdateAdministratorRequestCopyWith<$Res> {
  _$UpdateAdministratorRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of UpdateAdministratorRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = freezed,
    Object? password = freezed,
  }) {
    return _then(_value.copyWith(
      name: freezed == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String?,
      password: freezed == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$UpdateAdministratorRequestImplCopyWith<$Res>
    implements $UpdateAdministratorRequestCopyWith<$Res> {
  factory _$$UpdateAdministratorRequestImplCopyWith(
          _$UpdateAdministratorRequestImpl value,
          $Res Function(_$UpdateAdministratorRequestImpl) then) =
      __$$UpdateAdministratorRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String? name, String? password});
}

/// @nodoc
class __$$UpdateAdministratorRequestImplCopyWithImpl<$Res>
    extends _$UpdateAdministratorRequestCopyWithImpl<$Res,
        _$UpdateAdministratorRequestImpl>
    implements _$$UpdateAdministratorRequestImplCopyWith<$Res> {
  __$$UpdateAdministratorRequestImplCopyWithImpl(
      _$UpdateAdministratorRequestImpl _value,
      $Res Function(_$UpdateAdministratorRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of UpdateAdministratorRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = freezed,
    Object? password = freezed,
  }) {
    return _then(_$UpdateAdministratorRequestImpl(
      name: freezed == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String?,
      password: freezed == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$UpdateAdministratorRequestImpl implements _UpdateAdministratorRequest {
  const _$UpdateAdministratorRequestImpl({this.name, this.password});

  factory _$UpdateAdministratorRequestImpl.fromJson(
          Map<String, dynamic> json) =>
      _$$UpdateAdministratorRequestImplFromJson(json);

  @override
  final String? name;
  @override
  final String? password;

  @override
  String toString() {
    return 'UpdateAdministratorRequest(name: $name, password: $password)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UpdateAdministratorRequestImpl &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.password, password) ||
                other.password == password));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, name, password);

  /// Create a copy of UpdateAdministratorRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UpdateAdministratorRequestImplCopyWith<_$UpdateAdministratorRequestImpl>
      get copyWith => __$$UpdateAdministratorRequestImplCopyWithImpl<
          _$UpdateAdministratorRequestImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$UpdateAdministratorRequestImplToJson(
      this,
    );
  }
}

abstract class _UpdateAdministratorRequest
    implements UpdateAdministratorRequest {
  const factory _UpdateAdministratorRequest(
      {final String? name,
      final String? password}) = _$UpdateAdministratorRequestImpl;

  factory _UpdateAdministratorRequest.fromJson(Map<String, dynamic> json) =
      _$UpdateAdministratorRequestImpl.fromJson;

  @override
  String? get name;
  @override
  String? get password;

  /// Create a copy of UpdateAdministratorRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UpdateAdministratorRequestImplCopyWith<_$UpdateAdministratorRequestImpl>
      get copyWith => throw _privateConstructorUsedError;
}
