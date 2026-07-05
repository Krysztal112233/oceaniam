// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'key.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

ApplicationKey _$ApplicationKeyFromJson(Map<String, dynamic> json) {
  return _ApplicationKey.fromJson(json);
}

/// @nodoc
mixin _$ApplicationKey {
  @JsonKey(name: 'key_id')
  String get keyId => throw _privateConstructorUsedError;
  String get algorithm => throw _privateConstructorUsedError;
  String get status => throw _privateConstructorUsedError;
  @JsonKey(name: 'activated_at')
  String get activatedAt => throw _privateConstructorUsedError;

  /// Serializes this ApplicationKey to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ApplicationKey
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ApplicationKeyCopyWith<ApplicationKey> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ApplicationKeyCopyWith<$Res> {
  factory $ApplicationKeyCopyWith(
          ApplicationKey value, $Res Function(ApplicationKey) then) =
      _$ApplicationKeyCopyWithImpl<$Res, ApplicationKey>;
  @useResult
  $Res call(
      {@JsonKey(name: 'key_id') String keyId,
      String algorithm,
      String status,
      @JsonKey(name: 'activated_at') String activatedAt});
}

/// @nodoc
class _$ApplicationKeyCopyWithImpl<$Res, $Val extends ApplicationKey>
    implements $ApplicationKeyCopyWith<$Res> {
  _$ApplicationKeyCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ApplicationKey
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? keyId = null,
    Object? algorithm = null,
    Object? status = null,
    Object? activatedAt = null,
  }) {
    return _then(_value.copyWith(
      keyId: null == keyId
          ? _value.keyId
          : keyId // ignore: cast_nullable_to_non_nullable
              as String,
      algorithm: null == algorithm
          ? _value.algorithm
          : algorithm // ignore: cast_nullable_to_non_nullable
              as String,
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as String,
      activatedAt: null == activatedAt
          ? _value.activatedAt
          : activatedAt // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ApplicationKeyImplCopyWith<$Res>
    implements $ApplicationKeyCopyWith<$Res> {
  factory _$$ApplicationKeyImplCopyWith(_$ApplicationKeyImpl value,
          $Res Function(_$ApplicationKeyImpl) then) =
      __$$ApplicationKeyImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'key_id') String keyId,
      String algorithm,
      String status,
      @JsonKey(name: 'activated_at') String activatedAt});
}

/// @nodoc
class __$$ApplicationKeyImplCopyWithImpl<$Res>
    extends _$ApplicationKeyCopyWithImpl<$Res, _$ApplicationKeyImpl>
    implements _$$ApplicationKeyImplCopyWith<$Res> {
  __$$ApplicationKeyImplCopyWithImpl(
      _$ApplicationKeyImpl _value, $Res Function(_$ApplicationKeyImpl) _then)
      : super(_value, _then);

  /// Create a copy of ApplicationKey
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? keyId = null,
    Object? algorithm = null,
    Object? status = null,
    Object? activatedAt = null,
  }) {
    return _then(_$ApplicationKeyImpl(
      keyId: null == keyId
          ? _value.keyId
          : keyId // ignore: cast_nullable_to_non_nullable
              as String,
      algorithm: null == algorithm
          ? _value.algorithm
          : algorithm // ignore: cast_nullable_to_non_nullable
              as String,
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as String,
      activatedAt: null == activatedAt
          ? _value.activatedAt
          : activatedAt // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ApplicationKeyImpl implements _ApplicationKey {
  const _$ApplicationKeyImpl(
      {@JsonKey(name: 'key_id') required this.keyId,
      required this.algorithm,
      required this.status,
      @JsonKey(name: 'activated_at') required this.activatedAt});

  factory _$ApplicationKeyImpl.fromJson(Map<String, dynamic> json) =>
      _$$ApplicationKeyImplFromJson(json);

  @override
  @JsonKey(name: 'key_id')
  final String keyId;
  @override
  final String algorithm;
  @override
  final String status;
  @override
  @JsonKey(name: 'activated_at')
  final String activatedAt;

  @override
  String toString() {
    return 'ApplicationKey(keyId: $keyId, algorithm: $algorithm, status: $status, activatedAt: $activatedAt)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ApplicationKeyImpl &&
            (identical(other.keyId, keyId) || other.keyId == keyId) &&
            (identical(other.algorithm, algorithm) ||
                other.algorithm == algorithm) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.activatedAt, activatedAt) ||
                other.activatedAt == activatedAt));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, keyId, algorithm, status, activatedAt);

  /// Create a copy of ApplicationKey
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ApplicationKeyImplCopyWith<_$ApplicationKeyImpl> get copyWith =>
      __$$ApplicationKeyImplCopyWithImpl<_$ApplicationKeyImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ApplicationKeyImplToJson(
      this,
    );
  }
}

abstract class _ApplicationKey implements ApplicationKey {
  const factory _ApplicationKey(
          {@JsonKey(name: 'key_id') required final String keyId,
          required final String algorithm,
          required final String status,
          @JsonKey(name: 'activated_at') required final String activatedAt}) =
      _$ApplicationKeyImpl;

  factory _ApplicationKey.fromJson(Map<String, dynamic> json) =
      _$ApplicationKeyImpl.fromJson;

  @override
  @JsonKey(name: 'key_id')
  String get keyId;
  @override
  String get algorithm;
  @override
  String get status;
  @override
  @JsonKey(name: 'activated_at')
  String get activatedAt;

  /// Create a copy of ApplicationKey
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ApplicationKeyImplCopyWith<_$ApplicationKeyImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
