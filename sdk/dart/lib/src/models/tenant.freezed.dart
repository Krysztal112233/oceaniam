// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'tenant.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

Tenant _$TenantFromJson(Map<String, dynamic> json) {
  return _Tenant.fromJson(json);
}

/// @nodoc
mixin _$Tenant {
  String get id => throw _privateConstructorUsedError;
  String? get comment => throw _privateConstructorUsedError;

  /// Serializes this Tenant to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of Tenant
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $TenantCopyWith<Tenant> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $TenantCopyWith<$Res> {
  factory $TenantCopyWith(Tenant value, $Res Function(Tenant) then) =
      _$TenantCopyWithImpl<$Res, Tenant>;
  @useResult
  $Res call({String id, String? comment});
}

/// @nodoc
class _$TenantCopyWithImpl<$Res, $Val extends Tenant>
    implements $TenantCopyWith<$Res> {
  _$TenantCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Tenant
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? comment = freezed,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      comment: freezed == comment
          ? _value.comment
          : comment // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$TenantImplCopyWith<$Res> implements $TenantCopyWith<$Res> {
  factory _$$TenantImplCopyWith(
          _$TenantImpl value, $Res Function(_$TenantImpl) then) =
      __$$TenantImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String id, String? comment});
}

/// @nodoc
class __$$TenantImplCopyWithImpl<$Res>
    extends _$TenantCopyWithImpl<$Res, _$TenantImpl>
    implements _$$TenantImplCopyWith<$Res> {
  __$$TenantImplCopyWithImpl(
      _$TenantImpl _value, $Res Function(_$TenantImpl) _then)
      : super(_value, _then);

  /// Create a copy of Tenant
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? comment = freezed,
  }) {
    return _then(_$TenantImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      comment: freezed == comment
          ? _value.comment
          : comment // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$TenantImpl implements _Tenant {
  const _$TenantImpl({required this.id, this.comment});

  factory _$TenantImpl.fromJson(Map<String, dynamic> json) =>
      _$$TenantImplFromJson(json);

  @override
  final String id;
  @override
  final String? comment;

  @override
  String toString() {
    return 'Tenant(id: $id, comment: $comment)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$TenantImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.comment, comment) || other.comment == comment));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, comment);

  /// Create a copy of Tenant
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$TenantImplCopyWith<_$TenantImpl> get copyWith =>
      __$$TenantImplCopyWithImpl<_$TenantImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$TenantImplToJson(
      this,
    );
  }
}

abstract class _Tenant implements Tenant {
  const factory _Tenant({required final String id, final String? comment}) =
      _$TenantImpl;

  factory _Tenant.fromJson(Map<String, dynamic> json) = _$TenantImpl.fromJson;

  @override
  String get id;
  @override
  String? get comment;

  /// Create a copy of Tenant
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$TenantImplCopyWith<_$TenantImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

CreateTenantRequest _$CreateTenantRequestFromJson(Map<String, dynamic> json) {
  return _CreateTenantRequest.fromJson(json);
}

/// @nodoc
mixin _$CreateTenantRequest {
  String? get comment => throw _privateConstructorUsedError;

  /// Serializes this CreateTenantRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of CreateTenantRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $CreateTenantRequestCopyWith<CreateTenantRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $CreateTenantRequestCopyWith<$Res> {
  factory $CreateTenantRequestCopyWith(
          CreateTenantRequest value, $Res Function(CreateTenantRequest) then) =
      _$CreateTenantRequestCopyWithImpl<$Res, CreateTenantRequest>;
  @useResult
  $Res call({String? comment});
}

/// @nodoc
class _$CreateTenantRequestCopyWithImpl<$Res, $Val extends CreateTenantRequest>
    implements $CreateTenantRequestCopyWith<$Res> {
  _$CreateTenantRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of CreateTenantRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? comment = freezed,
  }) {
    return _then(_value.copyWith(
      comment: freezed == comment
          ? _value.comment
          : comment // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$CreateTenantRequestImplCopyWith<$Res>
    implements $CreateTenantRequestCopyWith<$Res> {
  factory _$$CreateTenantRequestImplCopyWith(_$CreateTenantRequestImpl value,
          $Res Function(_$CreateTenantRequestImpl) then) =
      __$$CreateTenantRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String? comment});
}

/// @nodoc
class __$$CreateTenantRequestImplCopyWithImpl<$Res>
    extends _$CreateTenantRequestCopyWithImpl<$Res, _$CreateTenantRequestImpl>
    implements _$$CreateTenantRequestImplCopyWith<$Res> {
  __$$CreateTenantRequestImplCopyWithImpl(_$CreateTenantRequestImpl _value,
      $Res Function(_$CreateTenantRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of CreateTenantRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? comment = freezed,
  }) {
    return _then(_$CreateTenantRequestImpl(
      comment: freezed == comment
          ? _value.comment
          : comment // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$CreateTenantRequestImpl implements _CreateTenantRequest {
  const _$CreateTenantRequestImpl({this.comment});

  factory _$CreateTenantRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$CreateTenantRequestImplFromJson(json);

  @override
  final String? comment;

  @override
  String toString() {
    return 'CreateTenantRequest(comment: $comment)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CreateTenantRequestImpl &&
            (identical(other.comment, comment) || other.comment == comment));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, comment);

  /// Create a copy of CreateTenantRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CreateTenantRequestImplCopyWith<_$CreateTenantRequestImpl> get copyWith =>
      __$$CreateTenantRequestImplCopyWithImpl<_$CreateTenantRequestImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$CreateTenantRequestImplToJson(
      this,
    );
  }
}

abstract class _CreateTenantRequest implements CreateTenantRequest {
  const factory _CreateTenantRequest({final String? comment}) =
      _$CreateTenantRequestImpl;

  factory _CreateTenantRequest.fromJson(Map<String, dynamic> json) =
      _$CreateTenantRequestImpl.fromJson;

  @override
  String? get comment;

  /// Create a copy of CreateTenantRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CreateTenantRequestImplCopyWith<_$CreateTenantRequestImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

UpdateTenantRequest _$UpdateTenantRequestFromJson(Map<String, dynamic> json) {
  return _UpdateTenantRequest.fromJson(json);
}

/// @nodoc
mixin _$UpdateTenantRequest {
  String? get comment => throw _privateConstructorUsedError;

  /// Serializes this UpdateTenantRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of UpdateTenantRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $UpdateTenantRequestCopyWith<UpdateTenantRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $UpdateTenantRequestCopyWith<$Res> {
  factory $UpdateTenantRequestCopyWith(
          UpdateTenantRequest value, $Res Function(UpdateTenantRequest) then) =
      _$UpdateTenantRequestCopyWithImpl<$Res, UpdateTenantRequest>;
  @useResult
  $Res call({String? comment});
}

/// @nodoc
class _$UpdateTenantRequestCopyWithImpl<$Res, $Val extends UpdateTenantRequest>
    implements $UpdateTenantRequestCopyWith<$Res> {
  _$UpdateTenantRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of UpdateTenantRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? comment = freezed,
  }) {
    return _then(_value.copyWith(
      comment: freezed == comment
          ? _value.comment
          : comment // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$UpdateTenantRequestImplCopyWith<$Res>
    implements $UpdateTenantRequestCopyWith<$Res> {
  factory _$$UpdateTenantRequestImplCopyWith(_$UpdateTenantRequestImpl value,
          $Res Function(_$UpdateTenantRequestImpl) then) =
      __$$UpdateTenantRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String? comment});
}

/// @nodoc
class __$$UpdateTenantRequestImplCopyWithImpl<$Res>
    extends _$UpdateTenantRequestCopyWithImpl<$Res, _$UpdateTenantRequestImpl>
    implements _$$UpdateTenantRequestImplCopyWith<$Res> {
  __$$UpdateTenantRequestImplCopyWithImpl(_$UpdateTenantRequestImpl _value,
      $Res Function(_$UpdateTenantRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of UpdateTenantRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? comment = freezed,
  }) {
    return _then(_$UpdateTenantRequestImpl(
      comment: freezed == comment
          ? _value.comment
          : comment // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$UpdateTenantRequestImpl implements _UpdateTenantRequest {
  const _$UpdateTenantRequestImpl({this.comment});

  factory _$UpdateTenantRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$UpdateTenantRequestImplFromJson(json);

  @override
  final String? comment;

  @override
  String toString() {
    return 'UpdateTenantRequest(comment: $comment)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UpdateTenantRequestImpl &&
            (identical(other.comment, comment) || other.comment == comment));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, comment);

  /// Create a copy of UpdateTenantRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UpdateTenantRequestImplCopyWith<_$UpdateTenantRequestImpl> get copyWith =>
      __$$UpdateTenantRequestImplCopyWithImpl<_$UpdateTenantRequestImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$UpdateTenantRequestImplToJson(
      this,
    );
  }
}

abstract class _UpdateTenantRequest implements UpdateTenantRequest {
  const factory _UpdateTenantRequest({final String? comment}) =
      _$UpdateTenantRequestImpl;

  factory _UpdateTenantRequest.fromJson(Map<String, dynamic> json) =
      _$UpdateTenantRequestImpl.fromJson;

  @override
  String? get comment;

  /// Create a copy of UpdateTenantRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UpdateTenantRequestImplCopyWith<_$UpdateTenantRequestImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
