// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'user.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

ApplicationUser _$ApplicationUserFromJson(Map<String, dynamic> json) {
  return _ApplicationUser.fromJson(json);
}

/// @nodoc
mixin _$ApplicationUser {
  String get id => throw _privateConstructorUsedError;
  String? get email => throw _privateConstructorUsedError;
  String? get phone => throw _privateConstructorUsedError;
  String get nickname => throw _privateConstructorUsedError;

  /// Serializes this ApplicationUser to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ApplicationUser
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ApplicationUserCopyWith<ApplicationUser> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ApplicationUserCopyWith<$Res> {
  factory $ApplicationUserCopyWith(
          ApplicationUser value, $Res Function(ApplicationUser) then) =
      _$ApplicationUserCopyWithImpl<$Res, ApplicationUser>;
  @useResult
  $Res call({String id, String? email, String? phone, String nickname});
}

/// @nodoc
class _$ApplicationUserCopyWithImpl<$Res, $Val extends ApplicationUser>
    implements $ApplicationUserCopyWith<$Res> {
  _$ApplicationUserCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ApplicationUser
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? email = freezed,
    Object? phone = freezed,
    Object? nickname = null,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      email: freezed == email
          ? _value.email
          : email // ignore: cast_nullable_to_non_nullable
              as String?,
      phone: freezed == phone
          ? _value.phone
          : phone // ignore: cast_nullable_to_non_nullable
              as String?,
      nickname: null == nickname
          ? _value.nickname
          : nickname // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ApplicationUserImplCopyWith<$Res>
    implements $ApplicationUserCopyWith<$Res> {
  factory _$$ApplicationUserImplCopyWith(_$ApplicationUserImpl value,
          $Res Function(_$ApplicationUserImpl) then) =
      __$$ApplicationUserImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String id, String? email, String? phone, String nickname});
}

/// @nodoc
class __$$ApplicationUserImplCopyWithImpl<$Res>
    extends _$ApplicationUserCopyWithImpl<$Res, _$ApplicationUserImpl>
    implements _$$ApplicationUserImplCopyWith<$Res> {
  __$$ApplicationUserImplCopyWithImpl(
      _$ApplicationUserImpl _value, $Res Function(_$ApplicationUserImpl) _then)
      : super(_value, _then);

  /// Create a copy of ApplicationUser
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? email = freezed,
    Object? phone = freezed,
    Object? nickname = null,
  }) {
    return _then(_$ApplicationUserImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      email: freezed == email
          ? _value.email
          : email // ignore: cast_nullable_to_non_nullable
              as String?,
      phone: freezed == phone
          ? _value.phone
          : phone // ignore: cast_nullable_to_non_nullable
              as String?,
      nickname: null == nickname
          ? _value.nickname
          : nickname // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ApplicationUserImpl implements _ApplicationUser {
  const _$ApplicationUserImpl(
      {required this.id, this.email, this.phone, required this.nickname});

  factory _$ApplicationUserImpl.fromJson(Map<String, dynamic> json) =>
      _$$ApplicationUserImplFromJson(json);

  @override
  final String id;
  @override
  final String? email;
  @override
  final String? phone;
  @override
  final String nickname;

  @override
  String toString() {
    return 'ApplicationUser(id: $id, email: $email, phone: $phone, nickname: $nickname)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ApplicationUserImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.email, email) || other.email == email) &&
            (identical(other.phone, phone) || other.phone == phone) &&
            (identical(other.nickname, nickname) ||
                other.nickname == nickname));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, email, phone, nickname);

  /// Create a copy of ApplicationUser
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ApplicationUserImplCopyWith<_$ApplicationUserImpl> get copyWith =>
      __$$ApplicationUserImplCopyWithImpl<_$ApplicationUserImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ApplicationUserImplToJson(
      this,
    );
  }
}

abstract class _ApplicationUser implements ApplicationUser {
  const factory _ApplicationUser(
      {required final String id,
      final String? email,
      final String? phone,
      required final String nickname}) = _$ApplicationUserImpl;

  factory _ApplicationUser.fromJson(Map<String, dynamic> json) =
      _$ApplicationUserImpl.fromJson;

  @override
  String get id;
  @override
  String? get email;
  @override
  String? get phone;
  @override
  String get nickname;

  /// Create a copy of ApplicationUser
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ApplicationUserImplCopyWith<_$ApplicationUserImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

CreateUserRequest _$CreateUserRequestFromJson(Map<String, dynamic> json) {
  return _CreateUserRequest.fromJson(json);
}

/// @nodoc
mixin _$CreateUserRequest {
  String get nickname => throw _privateConstructorUsedError;
  String get password => throw _privateConstructorUsedError;
  String? get email => throw _privateConstructorUsedError;
  String? get phone => throw _privateConstructorUsedError;

  /// Serializes this CreateUserRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of CreateUserRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $CreateUserRequestCopyWith<CreateUserRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $CreateUserRequestCopyWith<$Res> {
  factory $CreateUserRequestCopyWith(
          CreateUserRequest value, $Res Function(CreateUserRequest) then) =
      _$CreateUserRequestCopyWithImpl<$Res, CreateUserRequest>;
  @useResult
  $Res call({String nickname, String password, String? email, String? phone});
}

/// @nodoc
class _$CreateUserRequestCopyWithImpl<$Res, $Val extends CreateUserRequest>
    implements $CreateUserRequestCopyWith<$Res> {
  _$CreateUserRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of CreateUserRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nickname = null,
    Object? password = null,
    Object? email = freezed,
    Object? phone = freezed,
  }) {
    return _then(_value.copyWith(
      nickname: null == nickname
          ? _value.nickname
          : nickname // ignore: cast_nullable_to_non_nullable
              as String,
      password: null == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as String,
      email: freezed == email
          ? _value.email
          : email // ignore: cast_nullable_to_non_nullable
              as String?,
      phone: freezed == phone
          ? _value.phone
          : phone // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$CreateUserRequestImplCopyWith<$Res>
    implements $CreateUserRequestCopyWith<$Res> {
  factory _$$CreateUserRequestImplCopyWith(_$CreateUserRequestImpl value,
          $Res Function(_$CreateUserRequestImpl) then) =
      __$$CreateUserRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String nickname, String password, String? email, String? phone});
}

/// @nodoc
class __$$CreateUserRequestImplCopyWithImpl<$Res>
    extends _$CreateUserRequestCopyWithImpl<$Res, _$CreateUserRequestImpl>
    implements _$$CreateUserRequestImplCopyWith<$Res> {
  __$$CreateUserRequestImplCopyWithImpl(_$CreateUserRequestImpl _value,
      $Res Function(_$CreateUserRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of CreateUserRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nickname = null,
    Object? password = null,
    Object? email = freezed,
    Object? phone = freezed,
  }) {
    return _then(_$CreateUserRequestImpl(
      nickname: null == nickname
          ? _value.nickname
          : nickname // ignore: cast_nullable_to_non_nullable
              as String,
      password: null == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as String,
      email: freezed == email
          ? _value.email
          : email // ignore: cast_nullable_to_non_nullable
              as String?,
      phone: freezed == phone
          ? _value.phone
          : phone // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$CreateUserRequestImpl implements _CreateUserRequest {
  const _$CreateUserRequestImpl(
      {required this.nickname, required this.password, this.email, this.phone});

  factory _$CreateUserRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$CreateUserRequestImplFromJson(json);

  @override
  final String nickname;
  @override
  final String password;
  @override
  final String? email;
  @override
  final String? phone;

  @override
  String toString() {
    return 'CreateUserRequest(nickname: $nickname, password: $password, email: $email, phone: $phone)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CreateUserRequestImpl &&
            (identical(other.nickname, nickname) ||
                other.nickname == nickname) &&
            (identical(other.password, password) ||
                other.password == password) &&
            (identical(other.email, email) || other.email == email) &&
            (identical(other.phone, phone) || other.phone == phone));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, nickname, password, email, phone);

  /// Create a copy of CreateUserRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CreateUserRequestImplCopyWith<_$CreateUserRequestImpl> get copyWith =>
      __$$CreateUserRequestImplCopyWithImpl<_$CreateUserRequestImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$CreateUserRequestImplToJson(
      this,
    );
  }
}

abstract class _CreateUserRequest implements CreateUserRequest {
  const factory _CreateUserRequest(
      {required final String nickname,
      required final String password,
      final String? email,
      final String? phone}) = _$CreateUserRequestImpl;

  factory _CreateUserRequest.fromJson(Map<String, dynamic> json) =
      _$CreateUserRequestImpl.fromJson;

  @override
  String get nickname;
  @override
  String get password;
  @override
  String? get email;
  @override
  String? get phone;

  /// Create a copy of CreateUserRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CreateUserRequestImplCopyWith<_$CreateUserRequestImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

PatchUserRequest _$PatchUserRequestFromJson(Map<String, dynamic> json) {
  return _PatchUserRequest.fromJson(json);
}

/// @nodoc
mixin _$PatchUserRequest {
  String? get nickname => throw _privateConstructorUsedError;

  /// Serializes this PatchUserRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of PatchUserRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $PatchUserRequestCopyWith<PatchUserRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PatchUserRequestCopyWith<$Res> {
  factory $PatchUserRequestCopyWith(
          PatchUserRequest value, $Res Function(PatchUserRequest) then) =
      _$PatchUserRequestCopyWithImpl<$Res, PatchUserRequest>;
  @useResult
  $Res call({String? nickname});
}

/// @nodoc
class _$PatchUserRequestCopyWithImpl<$Res, $Val extends PatchUserRequest>
    implements $PatchUserRequestCopyWith<$Res> {
  _$PatchUserRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PatchUserRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nickname = freezed,
  }) {
    return _then(_value.copyWith(
      nickname: freezed == nickname
          ? _value.nickname
          : nickname // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$PatchUserRequestImplCopyWith<$Res>
    implements $PatchUserRequestCopyWith<$Res> {
  factory _$$PatchUserRequestImplCopyWith(_$PatchUserRequestImpl value,
          $Res Function(_$PatchUserRequestImpl) then) =
      __$$PatchUserRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String? nickname});
}

/// @nodoc
class __$$PatchUserRequestImplCopyWithImpl<$Res>
    extends _$PatchUserRequestCopyWithImpl<$Res, _$PatchUserRequestImpl>
    implements _$$PatchUserRequestImplCopyWith<$Res> {
  __$$PatchUserRequestImplCopyWithImpl(_$PatchUserRequestImpl _value,
      $Res Function(_$PatchUserRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of PatchUserRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nickname = freezed,
  }) {
    return _then(_$PatchUserRequestImpl(
      nickname: freezed == nickname
          ? _value.nickname
          : nickname // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$PatchUserRequestImpl implements _PatchUserRequest {
  const _$PatchUserRequestImpl({this.nickname});

  factory _$PatchUserRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$PatchUserRequestImplFromJson(json);

  @override
  final String? nickname;

  @override
  String toString() {
    return 'PatchUserRequest(nickname: $nickname)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PatchUserRequestImpl &&
            (identical(other.nickname, nickname) ||
                other.nickname == nickname));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, nickname);

  /// Create a copy of PatchUserRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PatchUserRequestImplCopyWith<_$PatchUserRequestImpl> get copyWith =>
      __$$PatchUserRequestImplCopyWithImpl<_$PatchUserRequestImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$PatchUserRequestImplToJson(
      this,
    );
  }
}

abstract class _PatchUserRequest implements PatchUserRequest {
  const factory _PatchUserRequest({final String? nickname}) =
      _$PatchUserRequestImpl;

  factory _PatchUserRequest.fromJson(Map<String, dynamic> json) =
      _$PatchUserRequestImpl.fromJson;

  @override
  String? get nickname;

  /// Create a copy of PatchUserRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PatchUserRequestImplCopyWith<_$PatchUserRequestImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

UpdatePasswordRequest _$UpdatePasswordRequestFromJson(
    Map<String, dynamic> json) {
  return _UpdatePasswordRequest.fromJson(json);
}

/// @nodoc
mixin _$UpdatePasswordRequest {
  String get password => throw _privateConstructorUsedError;

  /// Serializes this UpdatePasswordRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of UpdatePasswordRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $UpdatePasswordRequestCopyWith<UpdatePasswordRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $UpdatePasswordRequestCopyWith<$Res> {
  factory $UpdatePasswordRequestCopyWith(UpdatePasswordRequest value,
          $Res Function(UpdatePasswordRequest) then) =
      _$UpdatePasswordRequestCopyWithImpl<$Res, UpdatePasswordRequest>;
  @useResult
  $Res call({String password});
}

/// @nodoc
class _$UpdatePasswordRequestCopyWithImpl<$Res,
        $Val extends UpdatePasswordRequest>
    implements $UpdatePasswordRequestCopyWith<$Res> {
  _$UpdatePasswordRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of UpdatePasswordRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? password = null,
  }) {
    return _then(_value.copyWith(
      password: null == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$UpdatePasswordRequestImplCopyWith<$Res>
    implements $UpdatePasswordRequestCopyWith<$Res> {
  factory _$$UpdatePasswordRequestImplCopyWith(
          _$UpdatePasswordRequestImpl value,
          $Res Function(_$UpdatePasswordRequestImpl) then) =
      __$$UpdatePasswordRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String password});
}

/// @nodoc
class __$$UpdatePasswordRequestImplCopyWithImpl<$Res>
    extends _$UpdatePasswordRequestCopyWithImpl<$Res,
        _$UpdatePasswordRequestImpl>
    implements _$$UpdatePasswordRequestImplCopyWith<$Res> {
  __$$UpdatePasswordRequestImplCopyWithImpl(_$UpdatePasswordRequestImpl _value,
      $Res Function(_$UpdatePasswordRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of UpdatePasswordRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? password = null,
  }) {
    return _then(_$UpdatePasswordRequestImpl(
      password: null == password
          ? _value.password
          : password // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$UpdatePasswordRequestImpl implements _UpdatePasswordRequest {
  const _$UpdatePasswordRequestImpl({required this.password});

  factory _$UpdatePasswordRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$UpdatePasswordRequestImplFromJson(json);

  @override
  final String password;

  @override
  String toString() {
    return 'UpdatePasswordRequest(password: $password)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UpdatePasswordRequestImpl &&
            (identical(other.password, password) ||
                other.password == password));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, password);

  /// Create a copy of UpdatePasswordRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UpdatePasswordRequestImplCopyWith<_$UpdatePasswordRequestImpl>
      get copyWith => __$$UpdatePasswordRequestImplCopyWithImpl<
          _$UpdatePasswordRequestImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$UpdatePasswordRequestImplToJson(
      this,
    );
  }
}

abstract class _UpdatePasswordRequest implements UpdatePasswordRequest {
  const factory _UpdatePasswordRequest({required final String password}) =
      _$UpdatePasswordRequestImpl;

  factory _UpdatePasswordRequest.fromJson(Map<String, dynamic> json) =
      _$UpdatePasswordRequestImpl.fromJson;

  @override
  String get password;

  /// Create a copy of UpdatePasswordRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UpdatePasswordRequestImplCopyWith<_$UpdatePasswordRequestImpl>
      get copyWith => throw _privateConstructorUsedError;
}

SearchApplicationUsersQuery _$SearchApplicationUsersQueryFromJson(
    Map<String, dynamic> json) {
  return _SearchApplicationUsersQuery.fromJson(json);
}

/// @nodoc
mixin _$SearchApplicationUsersQuery {
  int get page => throw _privateConstructorUsedError;
  @JsonKey(name: 'per_page')
  int get perPage => throw _privateConstructorUsedError;
  @JsonKey(name: 'sort_order')
  String? get sortOrder => throw _privateConstructorUsedError;
  @JsonKey(name: 'by_nickname')
  String? get byNickname => throw _privateConstructorUsedError;
  @JsonKey(name: 'by_email')
  String? get byEmail => throw _privateConstructorUsedError;
  @JsonKey(name: 'by_phone')
  String? get byPhone => throw _privateConstructorUsedError;
  @JsonKey(name: 'by_id')
  String? get byId => throw _privateConstructorUsedError;

  /// Serializes this SearchApplicationUsersQuery to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of SearchApplicationUsersQuery
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $SearchApplicationUsersQueryCopyWith<SearchApplicationUsersQuery>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SearchApplicationUsersQueryCopyWith<$Res> {
  factory $SearchApplicationUsersQueryCopyWith(
          SearchApplicationUsersQuery value,
          $Res Function(SearchApplicationUsersQuery) then) =
      _$SearchApplicationUsersQueryCopyWithImpl<$Res,
          SearchApplicationUsersQuery>;
  @useResult
  $Res call(
      {int page,
      @JsonKey(name: 'per_page') int perPage,
      @JsonKey(name: 'sort_order') String? sortOrder,
      @JsonKey(name: 'by_nickname') String? byNickname,
      @JsonKey(name: 'by_email') String? byEmail,
      @JsonKey(name: 'by_phone') String? byPhone,
      @JsonKey(name: 'by_id') String? byId});
}

/// @nodoc
class _$SearchApplicationUsersQueryCopyWithImpl<$Res,
        $Val extends SearchApplicationUsersQuery>
    implements $SearchApplicationUsersQueryCopyWith<$Res> {
  _$SearchApplicationUsersQueryCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of SearchApplicationUsersQuery
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? page = null,
    Object? perPage = null,
    Object? sortOrder = freezed,
    Object? byNickname = freezed,
    Object? byEmail = freezed,
    Object? byPhone = freezed,
    Object? byId = freezed,
  }) {
    return _then(_value.copyWith(
      page: null == page
          ? _value.page
          : page // ignore: cast_nullable_to_non_nullable
              as int,
      perPage: null == perPage
          ? _value.perPage
          : perPage // ignore: cast_nullable_to_non_nullable
              as int,
      sortOrder: freezed == sortOrder
          ? _value.sortOrder
          : sortOrder // ignore: cast_nullable_to_non_nullable
              as String?,
      byNickname: freezed == byNickname
          ? _value.byNickname
          : byNickname // ignore: cast_nullable_to_non_nullable
              as String?,
      byEmail: freezed == byEmail
          ? _value.byEmail
          : byEmail // ignore: cast_nullable_to_non_nullable
              as String?,
      byPhone: freezed == byPhone
          ? _value.byPhone
          : byPhone // ignore: cast_nullable_to_non_nullable
              as String?,
      byId: freezed == byId
          ? _value.byId
          : byId // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SearchApplicationUsersQueryImplCopyWith<$Res>
    implements $SearchApplicationUsersQueryCopyWith<$Res> {
  factory _$$SearchApplicationUsersQueryImplCopyWith(
          _$SearchApplicationUsersQueryImpl value,
          $Res Function(_$SearchApplicationUsersQueryImpl) then) =
      __$$SearchApplicationUsersQueryImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {int page,
      @JsonKey(name: 'per_page') int perPage,
      @JsonKey(name: 'sort_order') String? sortOrder,
      @JsonKey(name: 'by_nickname') String? byNickname,
      @JsonKey(name: 'by_email') String? byEmail,
      @JsonKey(name: 'by_phone') String? byPhone,
      @JsonKey(name: 'by_id') String? byId});
}

/// @nodoc
class __$$SearchApplicationUsersQueryImplCopyWithImpl<$Res>
    extends _$SearchApplicationUsersQueryCopyWithImpl<$Res,
        _$SearchApplicationUsersQueryImpl>
    implements _$$SearchApplicationUsersQueryImplCopyWith<$Res> {
  __$$SearchApplicationUsersQueryImplCopyWithImpl(
      _$SearchApplicationUsersQueryImpl _value,
      $Res Function(_$SearchApplicationUsersQueryImpl) _then)
      : super(_value, _then);

  /// Create a copy of SearchApplicationUsersQuery
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? page = null,
    Object? perPage = null,
    Object? sortOrder = freezed,
    Object? byNickname = freezed,
    Object? byEmail = freezed,
    Object? byPhone = freezed,
    Object? byId = freezed,
  }) {
    return _then(_$SearchApplicationUsersQueryImpl(
      page: null == page
          ? _value.page
          : page // ignore: cast_nullable_to_non_nullable
              as int,
      perPage: null == perPage
          ? _value.perPage
          : perPage // ignore: cast_nullable_to_non_nullable
              as int,
      sortOrder: freezed == sortOrder
          ? _value.sortOrder
          : sortOrder // ignore: cast_nullable_to_non_nullable
              as String?,
      byNickname: freezed == byNickname
          ? _value.byNickname
          : byNickname // ignore: cast_nullable_to_non_nullable
              as String?,
      byEmail: freezed == byEmail
          ? _value.byEmail
          : byEmail // ignore: cast_nullable_to_non_nullable
              as String?,
      byPhone: freezed == byPhone
          ? _value.byPhone
          : byPhone // ignore: cast_nullable_to_non_nullable
              as String?,
      byId: freezed == byId
          ? _value.byId
          : byId // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SearchApplicationUsersQueryImpl
    implements _SearchApplicationUsersQuery {
  const _$SearchApplicationUsersQueryImpl(
      {this.page = 1,
      @JsonKey(name: 'per_page') this.perPage = 30,
      @JsonKey(name: 'sort_order') this.sortOrder,
      @JsonKey(name: 'by_nickname') this.byNickname,
      @JsonKey(name: 'by_email') this.byEmail,
      @JsonKey(name: 'by_phone') this.byPhone,
      @JsonKey(name: 'by_id') this.byId});

  factory _$SearchApplicationUsersQueryImpl.fromJson(
          Map<String, dynamic> json) =>
      _$$SearchApplicationUsersQueryImplFromJson(json);

  @override
  @JsonKey()
  final int page;
  @override
  @JsonKey(name: 'per_page')
  final int perPage;
  @override
  @JsonKey(name: 'sort_order')
  final String? sortOrder;
  @override
  @JsonKey(name: 'by_nickname')
  final String? byNickname;
  @override
  @JsonKey(name: 'by_email')
  final String? byEmail;
  @override
  @JsonKey(name: 'by_phone')
  final String? byPhone;
  @override
  @JsonKey(name: 'by_id')
  final String? byId;

  @override
  String toString() {
    return 'SearchApplicationUsersQuery(page: $page, perPage: $perPage, sortOrder: $sortOrder, byNickname: $byNickname, byEmail: $byEmail, byPhone: $byPhone, byId: $byId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SearchApplicationUsersQueryImpl &&
            (identical(other.page, page) || other.page == page) &&
            (identical(other.perPage, perPage) || other.perPage == perPage) &&
            (identical(other.sortOrder, sortOrder) ||
                other.sortOrder == sortOrder) &&
            (identical(other.byNickname, byNickname) ||
                other.byNickname == byNickname) &&
            (identical(other.byEmail, byEmail) || other.byEmail == byEmail) &&
            (identical(other.byPhone, byPhone) || other.byPhone == byPhone) &&
            (identical(other.byId, byId) || other.byId == byId));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, page, perPage, sortOrder,
      byNickname, byEmail, byPhone, byId);

  /// Create a copy of SearchApplicationUsersQuery
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SearchApplicationUsersQueryImplCopyWith<_$SearchApplicationUsersQueryImpl>
      get copyWith => __$$SearchApplicationUsersQueryImplCopyWithImpl<
          _$SearchApplicationUsersQueryImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SearchApplicationUsersQueryImplToJson(
      this,
    );
  }
}

abstract class _SearchApplicationUsersQuery
    implements SearchApplicationUsersQuery {
  const factory _SearchApplicationUsersQuery(
          {final int page,
          @JsonKey(name: 'per_page') final int perPage,
          @JsonKey(name: 'sort_order') final String? sortOrder,
          @JsonKey(name: 'by_nickname') final String? byNickname,
          @JsonKey(name: 'by_email') final String? byEmail,
          @JsonKey(name: 'by_phone') final String? byPhone,
          @JsonKey(name: 'by_id') final String? byId}) =
      _$SearchApplicationUsersQueryImpl;

  factory _SearchApplicationUsersQuery.fromJson(Map<String, dynamic> json) =
      _$SearchApplicationUsersQueryImpl.fromJson;

  @override
  int get page;
  @override
  @JsonKey(name: 'per_page')
  int get perPage;
  @override
  @JsonKey(name: 'sort_order')
  String? get sortOrder;
  @override
  @JsonKey(name: 'by_nickname')
  String? get byNickname;
  @override
  @JsonKey(name: 'by_email')
  String? get byEmail;
  @override
  @JsonKey(name: 'by_phone')
  String? get byPhone;
  @override
  @JsonKey(name: 'by_id')
  String? get byId;

  /// Create a copy of SearchApplicationUsersQuery
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SearchApplicationUsersQueryImplCopyWith<_$SearchApplicationUsersQueryImpl>
      get copyWith => throw _privateConstructorUsedError;
}
