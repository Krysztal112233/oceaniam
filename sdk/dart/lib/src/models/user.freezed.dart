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

UserSearchQuery _$UserSearchQueryFromJson(Map<String, dynamic> json) {
  return _UserSearchQuery.fromJson(json);
}

/// @nodoc
mixin _$UserSearchQuery {
  String? get query => throw _privateConstructorUsedError;
  @JsonKey(name: 'search_by')
  String? get searchBy => throw _privateConstructorUsedError;

  /// Serializes this UserSearchQuery to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of UserSearchQuery
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $UserSearchQueryCopyWith<UserSearchQuery> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $UserSearchQueryCopyWith<$Res> {
  factory $UserSearchQueryCopyWith(
          UserSearchQuery value, $Res Function(UserSearchQuery) then) =
      _$UserSearchQueryCopyWithImpl<$Res, UserSearchQuery>;
  @useResult
  $Res call({String? query, @JsonKey(name: 'search_by') String? searchBy});
}

/// @nodoc
class _$UserSearchQueryCopyWithImpl<$Res, $Val extends UserSearchQuery>
    implements $UserSearchQueryCopyWith<$Res> {
  _$UserSearchQueryCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of UserSearchQuery
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? query = freezed,
    Object? searchBy = freezed,
  }) {
    return _then(_value.copyWith(
      query: freezed == query
          ? _value.query
          : query // ignore: cast_nullable_to_non_nullable
              as String?,
      searchBy: freezed == searchBy
          ? _value.searchBy
          : searchBy // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$UserSearchQueryImplCopyWith<$Res>
    implements $UserSearchQueryCopyWith<$Res> {
  factory _$$UserSearchQueryImplCopyWith(_$UserSearchQueryImpl value,
          $Res Function(_$UserSearchQueryImpl) then) =
      __$$UserSearchQueryImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String? query, @JsonKey(name: 'search_by') String? searchBy});
}

/// @nodoc
class __$$UserSearchQueryImplCopyWithImpl<$Res>
    extends _$UserSearchQueryCopyWithImpl<$Res, _$UserSearchQueryImpl>
    implements _$$UserSearchQueryImplCopyWith<$Res> {
  __$$UserSearchQueryImplCopyWithImpl(
      _$UserSearchQueryImpl _value, $Res Function(_$UserSearchQueryImpl) _then)
      : super(_value, _then);

  /// Create a copy of UserSearchQuery
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? query = freezed,
    Object? searchBy = freezed,
  }) {
    return _then(_$UserSearchQueryImpl(
      query: freezed == query
          ? _value.query
          : query // ignore: cast_nullable_to_non_nullable
              as String?,
      searchBy: freezed == searchBy
          ? _value.searchBy
          : searchBy // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$UserSearchQueryImpl implements _UserSearchQuery {
  const _$UserSearchQueryImpl(
      {this.query, @JsonKey(name: 'search_by') this.searchBy});

  factory _$UserSearchQueryImpl.fromJson(Map<String, dynamic> json) =>
      _$$UserSearchQueryImplFromJson(json);

  @override
  final String? query;
  @override
  @JsonKey(name: 'search_by')
  final String? searchBy;

  @override
  String toString() {
    return 'UserSearchQuery(query: $query, searchBy: $searchBy)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UserSearchQueryImpl &&
            (identical(other.query, query) || other.query == query) &&
            (identical(other.searchBy, searchBy) ||
                other.searchBy == searchBy));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, query, searchBy);

  /// Create a copy of UserSearchQuery
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UserSearchQueryImplCopyWith<_$UserSearchQueryImpl> get copyWith =>
      __$$UserSearchQueryImplCopyWithImpl<_$UserSearchQueryImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$UserSearchQueryImplToJson(
      this,
    );
  }
}

abstract class _UserSearchQuery implements UserSearchQuery {
  const factory _UserSearchQuery(
          {final String? query,
          @JsonKey(name: 'search_by') final String? searchBy}) =
      _$UserSearchQueryImpl;

  factory _UserSearchQuery.fromJson(Map<String, dynamic> json) =
      _$UserSearchQueryImpl.fromJson;

  @override
  String? get query;
  @override
  @JsonKey(name: 'search_by')
  String? get searchBy;

  /// Create a copy of UserSearchQuery
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UserSearchQueryImplCopyWith<_$UserSearchQueryImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
