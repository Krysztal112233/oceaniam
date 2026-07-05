// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'application.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

Application _$ApplicationFromJson(Map<String, dynamic> json) {
  return _Application.fromJson(json);
}

/// @nodoc
mixin _$Application {
  String get id => throw _privateConstructorUsedError;
  String? get comment => throw _privateConstructorUsedError;
  @JsonKey(name: 'tenant_id')
  String get tenantId => throw _privateConstructorUsedError;

  /// Serializes this Application to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of Application
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ApplicationCopyWith<Application> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ApplicationCopyWith<$Res> {
  factory $ApplicationCopyWith(
          Application value, $Res Function(Application) then) =
      _$ApplicationCopyWithImpl<$Res, Application>;
  @useResult
  $Res call(
      {String id,
      String? comment,
      @JsonKey(name: 'tenant_id') String tenantId});
}

/// @nodoc
class _$ApplicationCopyWithImpl<$Res, $Val extends Application>
    implements $ApplicationCopyWith<$Res> {
  _$ApplicationCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Application
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? comment = freezed,
    Object? tenantId = null,
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
      tenantId: null == tenantId
          ? _value.tenantId
          : tenantId // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ApplicationImplCopyWith<$Res>
    implements $ApplicationCopyWith<$Res> {
  factory _$$ApplicationImplCopyWith(
          _$ApplicationImpl value, $Res Function(_$ApplicationImpl) then) =
      __$$ApplicationImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String id,
      String? comment,
      @JsonKey(name: 'tenant_id') String tenantId});
}

/// @nodoc
class __$$ApplicationImplCopyWithImpl<$Res>
    extends _$ApplicationCopyWithImpl<$Res, _$ApplicationImpl>
    implements _$$ApplicationImplCopyWith<$Res> {
  __$$ApplicationImplCopyWithImpl(
      _$ApplicationImpl _value, $Res Function(_$ApplicationImpl) _then)
      : super(_value, _then);

  /// Create a copy of Application
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? comment = freezed,
    Object? tenantId = null,
  }) {
    return _then(_$ApplicationImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      comment: freezed == comment
          ? _value.comment
          : comment // ignore: cast_nullable_to_non_nullable
              as String?,
      tenantId: null == tenantId
          ? _value.tenantId
          : tenantId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ApplicationImpl implements _Application {
  const _$ApplicationImpl(
      {required this.id,
      this.comment,
      @JsonKey(name: 'tenant_id') required this.tenantId});

  factory _$ApplicationImpl.fromJson(Map<String, dynamic> json) =>
      _$$ApplicationImplFromJson(json);

  @override
  final String id;
  @override
  final String? comment;
  @override
  @JsonKey(name: 'tenant_id')
  final String tenantId;

  @override
  String toString() {
    return 'Application(id: $id, comment: $comment, tenantId: $tenantId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ApplicationImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.comment, comment) || other.comment == comment) &&
            (identical(other.tenantId, tenantId) ||
                other.tenantId == tenantId));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, comment, tenantId);

  /// Create a copy of Application
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ApplicationImplCopyWith<_$ApplicationImpl> get copyWith =>
      __$$ApplicationImplCopyWithImpl<_$ApplicationImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ApplicationImplToJson(
      this,
    );
  }
}

abstract class _Application implements Application {
  const factory _Application(
          {required final String id,
          final String? comment,
          @JsonKey(name: 'tenant_id') required final String tenantId}) =
      _$ApplicationImpl;

  factory _Application.fromJson(Map<String, dynamic> json) =
      _$ApplicationImpl.fromJson;

  @override
  String get id;
  @override
  String? get comment;
  @override
  @JsonKey(name: 'tenant_id')
  String get tenantId;

  /// Create a copy of Application
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ApplicationImplCopyWith<_$ApplicationImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

ApplicationDetail _$ApplicationDetailFromJson(Map<String, dynamic> json) {
  return _ApplicationDetail.fromJson(json);
}

/// @nodoc
mixin _$ApplicationDetail {
  String get id => throw _privateConstructorUsedError;
  String? get comment => throw _privateConstructorUsedError;
  @JsonKey(name: 'tenant_id')
  String get tenantId => throw _privateConstructorUsedError;

  /// Serializes this ApplicationDetail to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ApplicationDetail
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ApplicationDetailCopyWith<ApplicationDetail> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ApplicationDetailCopyWith<$Res> {
  factory $ApplicationDetailCopyWith(
          ApplicationDetail value, $Res Function(ApplicationDetail) then) =
      _$ApplicationDetailCopyWithImpl<$Res, ApplicationDetail>;
  @useResult
  $Res call(
      {String id,
      String? comment,
      @JsonKey(name: 'tenant_id') String tenantId});
}

/// @nodoc
class _$ApplicationDetailCopyWithImpl<$Res, $Val extends ApplicationDetail>
    implements $ApplicationDetailCopyWith<$Res> {
  _$ApplicationDetailCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ApplicationDetail
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? comment = freezed,
    Object? tenantId = null,
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
      tenantId: null == tenantId
          ? _value.tenantId
          : tenantId // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ApplicationDetailImplCopyWith<$Res>
    implements $ApplicationDetailCopyWith<$Res> {
  factory _$$ApplicationDetailImplCopyWith(_$ApplicationDetailImpl value,
          $Res Function(_$ApplicationDetailImpl) then) =
      __$$ApplicationDetailImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String id,
      String? comment,
      @JsonKey(name: 'tenant_id') String tenantId});
}

/// @nodoc
class __$$ApplicationDetailImplCopyWithImpl<$Res>
    extends _$ApplicationDetailCopyWithImpl<$Res, _$ApplicationDetailImpl>
    implements _$$ApplicationDetailImplCopyWith<$Res> {
  __$$ApplicationDetailImplCopyWithImpl(_$ApplicationDetailImpl _value,
      $Res Function(_$ApplicationDetailImpl) _then)
      : super(_value, _then);

  /// Create a copy of ApplicationDetail
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? comment = freezed,
    Object? tenantId = null,
  }) {
    return _then(_$ApplicationDetailImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      comment: freezed == comment
          ? _value.comment
          : comment // ignore: cast_nullable_to_non_nullable
              as String?,
      tenantId: null == tenantId
          ? _value.tenantId
          : tenantId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ApplicationDetailImpl implements _ApplicationDetail {
  const _$ApplicationDetailImpl(
      {required this.id,
      this.comment,
      @JsonKey(name: 'tenant_id') required this.tenantId});

  factory _$ApplicationDetailImpl.fromJson(Map<String, dynamic> json) =>
      _$$ApplicationDetailImplFromJson(json);

  @override
  final String id;
  @override
  final String? comment;
  @override
  @JsonKey(name: 'tenant_id')
  final String tenantId;

  @override
  String toString() {
    return 'ApplicationDetail(id: $id, comment: $comment, tenantId: $tenantId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ApplicationDetailImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.comment, comment) || other.comment == comment) &&
            (identical(other.tenantId, tenantId) ||
                other.tenantId == tenantId));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, comment, tenantId);

  /// Create a copy of ApplicationDetail
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ApplicationDetailImplCopyWith<_$ApplicationDetailImpl> get copyWith =>
      __$$ApplicationDetailImplCopyWithImpl<_$ApplicationDetailImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ApplicationDetailImplToJson(
      this,
    );
  }
}

abstract class _ApplicationDetail implements ApplicationDetail {
  const factory _ApplicationDetail(
          {required final String id,
          final String? comment,
          @JsonKey(name: 'tenant_id') required final String tenantId}) =
      _$ApplicationDetailImpl;

  factory _ApplicationDetail.fromJson(Map<String, dynamic> json) =
      _$ApplicationDetailImpl.fromJson;

  @override
  String get id;
  @override
  String? get comment;
  @override
  @JsonKey(name: 'tenant_id')
  String get tenantId;

  /// Create a copy of ApplicationDetail
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ApplicationDetailImplCopyWith<_$ApplicationDetailImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

CreateApplicationRequest _$CreateApplicationRequestFromJson(
    Map<String, dynamic> json) {
  return _CreateApplicationRequest.fromJson(json);
}

/// @nodoc
mixin _$CreateApplicationRequest {
  String? get comment => throw _privateConstructorUsedError;

  /// Serializes this CreateApplicationRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of CreateApplicationRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $CreateApplicationRequestCopyWith<CreateApplicationRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $CreateApplicationRequestCopyWith<$Res> {
  factory $CreateApplicationRequestCopyWith(CreateApplicationRequest value,
          $Res Function(CreateApplicationRequest) then) =
      _$CreateApplicationRequestCopyWithImpl<$Res, CreateApplicationRequest>;
  @useResult
  $Res call({String? comment});
}

/// @nodoc
class _$CreateApplicationRequestCopyWithImpl<$Res,
        $Val extends CreateApplicationRequest>
    implements $CreateApplicationRequestCopyWith<$Res> {
  _$CreateApplicationRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of CreateApplicationRequest
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
abstract class _$$CreateApplicationRequestImplCopyWith<$Res>
    implements $CreateApplicationRequestCopyWith<$Res> {
  factory _$$CreateApplicationRequestImplCopyWith(
          _$CreateApplicationRequestImpl value,
          $Res Function(_$CreateApplicationRequestImpl) then) =
      __$$CreateApplicationRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String? comment});
}

/// @nodoc
class __$$CreateApplicationRequestImplCopyWithImpl<$Res>
    extends _$CreateApplicationRequestCopyWithImpl<$Res,
        _$CreateApplicationRequestImpl>
    implements _$$CreateApplicationRequestImplCopyWith<$Res> {
  __$$CreateApplicationRequestImplCopyWithImpl(
      _$CreateApplicationRequestImpl _value,
      $Res Function(_$CreateApplicationRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of CreateApplicationRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? comment = freezed,
  }) {
    return _then(_$CreateApplicationRequestImpl(
      comment: freezed == comment
          ? _value.comment
          : comment // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$CreateApplicationRequestImpl implements _CreateApplicationRequest {
  const _$CreateApplicationRequestImpl({this.comment});

  factory _$CreateApplicationRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$CreateApplicationRequestImplFromJson(json);

  @override
  final String? comment;

  @override
  String toString() {
    return 'CreateApplicationRequest(comment: $comment)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CreateApplicationRequestImpl &&
            (identical(other.comment, comment) || other.comment == comment));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, comment);

  /// Create a copy of CreateApplicationRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CreateApplicationRequestImplCopyWith<_$CreateApplicationRequestImpl>
      get copyWith => __$$CreateApplicationRequestImplCopyWithImpl<
          _$CreateApplicationRequestImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$CreateApplicationRequestImplToJson(
      this,
    );
  }
}

abstract class _CreateApplicationRequest implements CreateApplicationRequest {
  const factory _CreateApplicationRequest({final String? comment}) =
      _$CreateApplicationRequestImpl;

  factory _CreateApplicationRequest.fromJson(Map<String, dynamic> json) =
      _$CreateApplicationRequestImpl.fromJson;

  @override
  String? get comment;

  /// Create a copy of CreateApplicationRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CreateApplicationRequestImplCopyWith<_$CreateApplicationRequestImpl>
      get copyWith => throw _privateConstructorUsedError;
}

UpdateApplicationRequest _$UpdateApplicationRequestFromJson(
    Map<String, dynamic> json) {
  return _UpdateApplicationRequest.fromJson(json);
}

/// @nodoc
mixin _$UpdateApplicationRequest {
  String? get comment => throw _privateConstructorUsedError;

  /// Serializes this UpdateApplicationRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of UpdateApplicationRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $UpdateApplicationRequestCopyWith<UpdateApplicationRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $UpdateApplicationRequestCopyWith<$Res> {
  factory $UpdateApplicationRequestCopyWith(UpdateApplicationRequest value,
          $Res Function(UpdateApplicationRequest) then) =
      _$UpdateApplicationRequestCopyWithImpl<$Res, UpdateApplicationRequest>;
  @useResult
  $Res call({String? comment});
}

/// @nodoc
class _$UpdateApplicationRequestCopyWithImpl<$Res,
        $Val extends UpdateApplicationRequest>
    implements $UpdateApplicationRequestCopyWith<$Res> {
  _$UpdateApplicationRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of UpdateApplicationRequest
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
abstract class _$$UpdateApplicationRequestImplCopyWith<$Res>
    implements $UpdateApplicationRequestCopyWith<$Res> {
  factory _$$UpdateApplicationRequestImplCopyWith(
          _$UpdateApplicationRequestImpl value,
          $Res Function(_$UpdateApplicationRequestImpl) then) =
      __$$UpdateApplicationRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String? comment});
}

/// @nodoc
class __$$UpdateApplicationRequestImplCopyWithImpl<$Res>
    extends _$UpdateApplicationRequestCopyWithImpl<$Res,
        _$UpdateApplicationRequestImpl>
    implements _$$UpdateApplicationRequestImplCopyWith<$Res> {
  __$$UpdateApplicationRequestImplCopyWithImpl(
      _$UpdateApplicationRequestImpl _value,
      $Res Function(_$UpdateApplicationRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of UpdateApplicationRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? comment = freezed,
  }) {
    return _then(_$UpdateApplicationRequestImpl(
      comment: freezed == comment
          ? _value.comment
          : comment // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$UpdateApplicationRequestImpl implements _UpdateApplicationRequest {
  const _$UpdateApplicationRequestImpl({this.comment});

  factory _$UpdateApplicationRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$UpdateApplicationRequestImplFromJson(json);

  @override
  final String? comment;

  @override
  String toString() {
    return 'UpdateApplicationRequest(comment: $comment)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UpdateApplicationRequestImpl &&
            (identical(other.comment, comment) || other.comment == comment));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, comment);

  /// Create a copy of UpdateApplicationRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UpdateApplicationRequestImplCopyWith<_$UpdateApplicationRequestImpl>
      get copyWith => __$$UpdateApplicationRequestImplCopyWithImpl<
          _$UpdateApplicationRequestImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$UpdateApplicationRequestImplToJson(
      this,
    );
  }
}

abstract class _UpdateApplicationRequest implements UpdateApplicationRequest {
  const factory _UpdateApplicationRequest({final String? comment}) =
      _$UpdateApplicationRequestImpl;

  factory _UpdateApplicationRequest.fromJson(Map<String, dynamic> json) =
      _$UpdateApplicationRequestImpl.fromJson;

  @override
  String? get comment;

  /// Create a copy of UpdateApplicationRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UpdateApplicationRequestImplCopyWith<_$UpdateApplicationRequestImpl>
      get copyWith => throw _privateConstructorUsedError;
}
