// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'role.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

ApplicationRole _$ApplicationRoleFromJson(Map<String, dynamic> json) {
  return _ApplicationRole.fromJson(json);
}

/// @nodoc
mixin _$ApplicationRole {
  String get id => throw _privateConstructorUsedError;
  String get name => throw _privateConstructorUsedError;
  @JsonKey(name: 'is_system')
  bool get isSystem => throw _privateConstructorUsedError;
  List<String> get permissions => throw _privateConstructorUsedError;

  /// Serializes this ApplicationRole to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ApplicationRole
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ApplicationRoleCopyWith<ApplicationRole> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ApplicationRoleCopyWith<$Res> {
  factory $ApplicationRoleCopyWith(
          ApplicationRole value, $Res Function(ApplicationRole) then) =
      _$ApplicationRoleCopyWithImpl<$Res, ApplicationRole>;
  @useResult
  $Res call(
      {String id,
      String name,
      @JsonKey(name: 'is_system') bool isSystem,
      List<String> permissions});
}

/// @nodoc
class _$ApplicationRoleCopyWithImpl<$Res, $Val extends ApplicationRole>
    implements $ApplicationRoleCopyWith<$Res> {
  _$ApplicationRoleCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ApplicationRole
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? isSystem = null,
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
      isSystem: null == isSystem
          ? _value.isSystem
          : isSystem // ignore: cast_nullable_to_non_nullable
              as bool,
      permissions: null == permissions
          ? _value.permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ApplicationRoleImplCopyWith<$Res>
    implements $ApplicationRoleCopyWith<$Res> {
  factory _$$ApplicationRoleImplCopyWith(_$ApplicationRoleImpl value,
          $Res Function(_$ApplicationRoleImpl) then) =
      __$$ApplicationRoleImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String id,
      String name,
      @JsonKey(name: 'is_system') bool isSystem,
      List<String> permissions});
}

/// @nodoc
class __$$ApplicationRoleImplCopyWithImpl<$Res>
    extends _$ApplicationRoleCopyWithImpl<$Res, _$ApplicationRoleImpl>
    implements _$$ApplicationRoleImplCopyWith<$Res> {
  __$$ApplicationRoleImplCopyWithImpl(
      _$ApplicationRoleImpl _value, $Res Function(_$ApplicationRoleImpl) _then)
      : super(_value, _then);

  /// Create a copy of ApplicationRole
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? isSystem = null,
    Object? permissions = null,
  }) {
    return _then(_$ApplicationRoleImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      isSystem: null == isSystem
          ? _value.isSystem
          : isSystem // ignore: cast_nullable_to_non_nullable
              as bool,
      permissions: null == permissions
          ? _value._permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ApplicationRoleImpl implements _ApplicationRole {
  const _$ApplicationRoleImpl(
      {required this.id,
      required this.name,
      @JsonKey(name: 'is_system') required this.isSystem,
      required final List<String> permissions})
      : _permissions = permissions;

  factory _$ApplicationRoleImpl.fromJson(Map<String, dynamic> json) =>
      _$$ApplicationRoleImplFromJson(json);

  @override
  final String id;
  @override
  final String name;
  @override
  @JsonKey(name: 'is_system')
  final bool isSystem;
  final List<String> _permissions;
  @override
  List<String> get permissions {
    if (_permissions is EqualUnmodifiableListView) return _permissions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_permissions);
  }

  @override
  String toString() {
    return 'ApplicationRole(id: $id, name: $name, isSystem: $isSystem, permissions: $permissions)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ApplicationRoleImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.isSystem, isSystem) ||
                other.isSystem == isSystem) &&
            const DeepCollectionEquality()
                .equals(other._permissions, _permissions));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, id, name, isSystem,
      const DeepCollectionEquality().hash(_permissions));

  /// Create a copy of ApplicationRole
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ApplicationRoleImplCopyWith<_$ApplicationRoleImpl> get copyWith =>
      __$$ApplicationRoleImplCopyWithImpl<_$ApplicationRoleImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ApplicationRoleImplToJson(
      this,
    );
  }
}

abstract class _ApplicationRole implements ApplicationRole {
  const factory _ApplicationRole(
      {required final String id,
      required final String name,
      @JsonKey(name: 'is_system') required final bool isSystem,
      required final List<String> permissions}) = _$ApplicationRoleImpl;

  factory _ApplicationRole.fromJson(Map<String, dynamic> json) =
      _$ApplicationRoleImpl.fromJson;

  @override
  String get id;
  @override
  String get name;
  @override
  @JsonKey(name: 'is_system')
  bool get isSystem;
  @override
  List<String> get permissions;

  /// Create a copy of ApplicationRole
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ApplicationRoleImplCopyWith<_$ApplicationRoleImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SubjectRoles _$SubjectRolesFromJson(Map<String, dynamic> json) {
  return _SubjectRoles.fromJson(json);
}

/// @nodoc
mixin _$SubjectRoles {
  @JsonKey(name: 'subject_id')
  String get subjectId => throw _privateConstructorUsedError;
  @JsonKey(name: 'role_ids')
  List<String> get roleIds => throw _privateConstructorUsedError;

  /// Serializes this SubjectRoles to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of SubjectRoles
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $SubjectRolesCopyWith<SubjectRoles> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SubjectRolesCopyWith<$Res> {
  factory $SubjectRolesCopyWith(
          SubjectRoles value, $Res Function(SubjectRoles) then) =
      _$SubjectRolesCopyWithImpl<$Res, SubjectRoles>;
  @useResult
  $Res call(
      {@JsonKey(name: 'subject_id') String subjectId,
      @JsonKey(name: 'role_ids') List<String> roleIds});
}

/// @nodoc
class _$SubjectRolesCopyWithImpl<$Res, $Val extends SubjectRoles>
    implements $SubjectRolesCopyWith<$Res> {
  _$SubjectRolesCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of SubjectRoles
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? subjectId = null,
    Object? roleIds = null,
  }) {
    return _then(_value.copyWith(
      subjectId: null == subjectId
          ? _value.subjectId
          : subjectId // ignore: cast_nullable_to_non_nullable
              as String,
      roleIds: null == roleIds
          ? _value.roleIds
          : roleIds // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SubjectRolesImplCopyWith<$Res>
    implements $SubjectRolesCopyWith<$Res> {
  factory _$$SubjectRolesImplCopyWith(
          _$SubjectRolesImpl value, $Res Function(_$SubjectRolesImpl) then) =
      __$$SubjectRolesImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'subject_id') String subjectId,
      @JsonKey(name: 'role_ids') List<String> roleIds});
}

/// @nodoc
class __$$SubjectRolesImplCopyWithImpl<$Res>
    extends _$SubjectRolesCopyWithImpl<$Res, _$SubjectRolesImpl>
    implements _$$SubjectRolesImplCopyWith<$Res> {
  __$$SubjectRolesImplCopyWithImpl(
      _$SubjectRolesImpl _value, $Res Function(_$SubjectRolesImpl) _then)
      : super(_value, _then);

  /// Create a copy of SubjectRoles
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? subjectId = null,
    Object? roleIds = null,
  }) {
    return _then(_$SubjectRolesImpl(
      subjectId: null == subjectId
          ? _value.subjectId
          : subjectId // ignore: cast_nullable_to_non_nullable
              as String,
      roleIds: null == roleIds
          ? _value._roleIds
          : roleIds // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SubjectRolesImpl implements _SubjectRoles {
  const _$SubjectRolesImpl(
      {@JsonKey(name: 'subject_id') required this.subjectId,
      @JsonKey(name: 'role_ids') required final List<String> roleIds})
      : _roleIds = roleIds;

  factory _$SubjectRolesImpl.fromJson(Map<String, dynamic> json) =>
      _$$SubjectRolesImplFromJson(json);

  @override
  @JsonKey(name: 'subject_id')
  final String subjectId;
  final List<String> _roleIds;
  @override
  @JsonKey(name: 'role_ids')
  List<String> get roleIds {
    if (_roleIds is EqualUnmodifiableListView) return _roleIds;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_roleIds);
  }

  @override
  String toString() {
    return 'SubjectRoles(subjectId: $subjectId, roleIds: $roleIds)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SubjectRolesImpl &&
            (identical(other.subjectId, subjectId) ||
                other.subjectId == subjectId) &&
            const DeepCollectionEquality().equals(other._roleIds, _roleIds));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType, subjectId, const DeepCollectionEquality().hash(_roleIds));

  /// Create a copy of SubjectRoles
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SubjectRolesImplCopyWith<_$SubjectRolesImpl> get copyWith =>
      __$$SubjectRolesImplCopyWithImpl<_$SubjectRolesImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SubjectRolesImplToJson(
      this,
    );
  }
}

abstract class _SubjectRoles implements SubjectRoles {
  const factory _SubjectRoles(
          {@JsonKey(name: 'subject_id') required final String subjectId,
          @JsonKey(name: 'role_ids') required final List<String> roleIds}) =
      _$SubjectRolesImpl;

  factory _SubjectRoles.fromJson(Map<String, dynamic> json) =
      _$SubjectRolesImpl.fromJson;

  @override
  @JsonKey(name: 'subject_id')
  String get subjectId;
  @override
  @JsonKey(name: 'role_ids')
  List<String> get roleIds;

  /// Create a copy of SubjectRoles
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SubjectRolesImplCopyWith<_$SubjectRolesImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

RolePermissions _$RolePermissionsFromJson(Map<String, dynamic> json) {
  return _RolePermissions.fromJson(json);
}

/// @nodoc
mixin _$RolePermissions {
  List<String> get permissions => throw _privateConstructorUsedError;

  /// Serializes this RolePermissions to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of RolePermissions
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $RolePermissionsCopyWith<RolePermissions> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $RolePermissionsCopyWith<$Res> {
  factory $RolePermissionsCopyWith(
          RolePermissions value, $Res Function(RolePermissions) then) =
      _$RolePermissionsCopyWithImpl<$Res, RolePermissions>;
  @useResult
  $Res call({List<String> permissions});
}

/// @nodoc
class _$RolePermissionsCopyWithImpl<$Res, $Val extends RolePermissions>
    implements $RolePermissionsCopyWith<$Res> {
  _$RolePermissionsCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of RolePermissions
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? permissions = null,
  }) {
    return _then(_value.copyWith(
      permissions: null == permissions
          ? _value.permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$RolePermissionsImplCopyWith<$Res>
    implements $RolePermissionsCopyWith<$Res> {
  factory _$$RolePermissionsImplCopyWith(_$RolePermissionsImpl value,
          $Res Function(_$RolePermissionsImpl) then) =
      __$$RolePermissionsImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({List<String> permissions});
}

/// @nodoc
class __$$RolePermissionsImplCopyWithImpl<$Res>
    extends _$RolePermissionsCopyWithImpl<$Res, _$RolePermissionsImpl>
    implements _$$RolePermissionsImplCopyWith<$Res> {
  __$$RolePermissionsImplCopyWithImpl(
      _$RolePermissionsImpl _value, $Res Function(_$RolePermissionsImpl) _then)
      : super(_value, _then);

  /// Create a copy of RolePermissions
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? permissions = null,
  }) {
    return _then(_$RolePermissionsImpl(
      permissions: null == permissions
          ? _value._permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$RolePermissionsImpl implements _RolePermissions {
  const _$RolePermissionsImpl({required final List<String> permissions})
      : _permissions = permissions;

  factory _$RolePermissionsImpl.fromJson(Map<String, dynamic> json) =>
      _$$RolePermissionsImplFromJson(json);

  final List<String> _permissions;
  @override
  List<String> get permissions {
    if (_permissions is EqualUnmodifiableListView) return _permissions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_permissions);
  }

  @override
  String toString() {
    return 'RolePermissions(permissions: $permissions)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$RolePermissionsImpl &&
            const DeepCollectionEquality()
                .equals(other._permissions, _permissions));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType, const DeepCollectionEquality().hash(_permissions));

  /// Create a copy of RolePermissions
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$RolePermissionsImplCopyWith<_$RolePermissionsImpl> get copyWith =>
      __$$RolePermissionsImplCopyWithImpl<_$RolePermissionsImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$RolePermissionsImplToJson(
      this,
    );
  }
}

abstract class _RolePermissions implements RolePermissions {
  const factory _RolePermissions({required final List<String> permissions}) =
      _$RolePermissionsImpl;

  factory _RolePermissions.fromJson(Map<String, dynamic> json) =
      _$RolePermissionsImpl.fromJson;

  @override
  List<String> get permissions;

  /// Create a copy of RolePermissions
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$RolePermissionsImplCopyWith<_$RolePermissionsImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

CreateRoleRequest _$CreateRoleRequestFromJson(Map<String, dynamic> json) {
  return _CreateRoleRequest.fromJson(json);
}

/// @nodoc
mixin _$CreateRoleRequest {
  String get name => throw _privateConstructorUsedError;
  List<String> get permissions => throw _privateConstructorUsedError;

  /// Serializes this CreateRoleRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of CreateRoleRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $CreateRoleRequestCopyWith<CreateRoleRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $CreateRoleRequestCopyWith<$Res> {
  factory $CreateRoleRequestCopyWith(
          CreateRoleRequest value, $Res Function(CreateRoleRequest) then) =
      _$CreateRoleRequestCopyWithImpl<$Res, CreateRoleRequest>;
  @useResult
  $Res call({String name, List<String> permissions});
}

/// @nodoc
class _$CreateRoleRequestCopyWithImpl<$Res, $Val extends CreateRoleRequest>
    implements $CreateRoleRequestCopyWith<$Res> {
  _$CreateRoleRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of CreateRoleRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = null,
    Object? permissions = null,
  }) {
    return _then(_value.copyWith(
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      permissions: null == permissions
          ? _value.permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$CreateRoleRequestImplCopyWith<$Res>
    implements $CreateRoleRequestCopyWith<$Res> {
  factory _$$CreateRoleRequestImplCopyWith(_$CreateRoleRequestImpl value,
          $Res Function(_$CreateRoleRequestImpl) then) =
      __$$CreateRoleRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String name, List<String> permissions});
}

/// @nodoc
class __$$CreateRoleRequestImplCopyWithImpl<$Res>
    extends _$CreateRoleRequestCopyWithImpl<$Res, _$CreateRoleRequestImpl>
    implements _$$CreateRoleRequestImplCopyWith<$Res> {
  __$$CreateRoleRequestImplCopyWithImpl(_$CreateRoleRequestImpl _value,
      $Res Function(_$CreateRoleRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of CreateRoleRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = null,
    Object? permissions = null,
  }) {
    return _then(_$CreateRoleRequestImpl(
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      permissions: null == permissions
          ? _value._permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$CreateRoleRequestImpl implements _CreateRoleRequest {
  const _$CreateRoleRequestImpl(
      {required this.name, required final List<String> permissions})
      : _permissions = permissions;

  factory _$CreateRoleRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$CreateRoleRequestImplFromJson(json);

  @override
  final String name;
  final List<String> _permissions;
  @override
  List<String> get permissions {
    if (_permissions is EqualUnmodifiableListView) return _permissions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_permissions);
  }

  @override
  String toString() {
    return 'CreateRoleRequest(name: $name, permissions: $permissions)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$CreateRoleRequestImpl &&
            (identical(other.name, name) || other.name == name) &&
            const DeepCollectionEquality()
                .equals(other._permissions, _permissions));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType, name, const DeepCollectionEquality().hash(_permissions));

  /// Create a copy of CreateRoleRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$CreateRoleRequestImplCopyWith<_$CreateRoleRequestImpl> get copyWith =>
      __$$CreateRoleRequestImplCopyWithImpl<_$CreateRoleRequestImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$CreateRoleRequestImplToJson(
      this,
    );
  }
}

abstract class _CreateRoleRequest implements CreateRoleRequest {
  const factory _CreateRoleRequest(
      {required final String name,
      required final List<String> permissions}) = _$CreateRoleRequestImpl;

  factory _CreateRoleRequest.fromJson(Map<String, dynamic> json) =
      _$CreateRoleRequestImpl.fromJson;

  @override
  String get name;
  @override
  List<String> get permissions;

  /// Create a copy of CreateRoleRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$CreateRoleRequestImplCopyWith<_$CreateRoleRequestImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

UpdateRoleRequest _$UpdateRoleRequestFromJson(Map<String, dynamic> json) {
  return _UpdateRoleRequest.fromJson(json);
}

/// @nodoc
mixin _$UpdateRoleRequest {
  String? get name => throw _privateConstructorUsedError;

  /// Serializes this UpdateRoleRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of UpdateRoleRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $UpdateRoleRequestCopyWith<UpdateRoleRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $UpdateRoleRequestCopyWith<$Res> {
  factory $UpdateRoleRequestCopyWith(
          UpdateRoleRequest value, $Res Function(UpdateRoleRequest) then) =
      _$UpdateRoleRequestCopyWithImpl<$Res, UpdateRoleRequest>;
  @useResult
  $Res call({String? name});
}

/// @nodoc
class _$UpdateRoleRequestCopyWithImpl<$Res, $Val extends UpdateRoleRequest>
    implements $UpdateRoleRequestCopyWith<$Res> {
  _$UpdateRoleRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of UpdateRoleRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = freezed,
  }) {
    return _then(_value.copyWith(
      name: freezed == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$UpdateRoleRequestImplCopyWith<$Res>
    implements $UpdateRoleRequestCopyWith<$Res> {
  factory _$$UpdateRoleRequestImplCopyWith(_$UpdateRoleRequestImpl value,
          $Res Function(_$UpdateRoleRequestImpl) then) =
      __$$UpdateRoleRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String? name});
}

/// @nodoc
class __$$UpdateRoleRequestImplCopyWithImpl<$Res>
    extends _$UpdateRoleRequestCopyWithImpl<$Res, _$UpdateRoleRequestImpl>
    implements _$$UpdateRoleRequestImplCopyWith<$Res> {
  __$$UpdateRoleRequestImplCopyWithImpl(_$UpdateRoleRequestImpl _value,
      $Res Function(_$UpdateRoleRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of UpdateRoleRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = freezed,
  }) {
    return _then(_$UpdateRoleRequestImpl(
      name: freezed == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$UpdateRoleRequestImpl implements _UpdateRoleRequest {
  const _$UpdateRoleRequestImpl({this.name});

  factory _$UpdateRoleRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$UpdateRoleRequestImplFromJson(json);

  @override
  final String? name;

  @override
  String toString() {
    return 'UpdateRoleRequest(name: $name)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UpdateRoleRequestImpl &&
            (identical(other.name, name) || other.name == name));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, name);

  /// Create a copy of UpdateRoleRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UpdateRoleRequestImplCopyWith<_$UpdateRoleRequestImpl> get copyWith =>
      __$$UpdateRoleRequestImplCopyWithImpl<_$UpdateRoleRequestImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$UpdateRoleRequestImplToJson(
      this,
    );
  }
}

abstract class _UpdateRoleRequest implements UpdateRoleRequest {
  const factory _UpdateRoleRequest({final String? name}) =
      _$UpdateRoleRequestImpl;

  factory _UpdateRoleRequest.fromJson(Map<String, dynamic> json) =
      _$UpdateRoleRequestImpl.fromJson;

  @override
  String? get name;

  /// Create a copy of UpdateRoleRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UpdateRoleRequestImplCopyWith<_$UpdateRoleRequestImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

UpdateRolePermissionsRequest _$UpdateRolePermissionsRequestFromJson(
    Map<String, dynamic> json) {
  return _UpdateRolePermissionsRequest.fromJson(json);
}

/// @nodoc
mixin _$UpdateRolePermissionsRequest {
  List<String> get permissions => throw _privateConstructorUsedError;

  /// Serializes this UpdateRolePermissionsRequest to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of UpdateRolePermissionsRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $UpdateRolePermissionsRequestCopyWith<UpdateRolePermissionsRequest>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $UpdateRolePermissionsRequestCopyWith<$Res> {
  factory $UpdateRolePermissionsRequestCopyWith(
          UpdateRolePermissionsRequest value,
          $Res Function(UpdateRolePermissionsRequest) then) =
      _$UpdateRolePermissionsRequestCopyWithImpl<$Res,
          UpdateRolePermissionsRequest>;
  @useResult
  $Res call({List<String> permissions});
}

/// @nodoc
class _$UpdateRolePermissionsRequestCopyWithImpl<$Res,
        $Val extends UpdateRolePermissionsRequest>
    implements $UpdateRolePermissionsRequestCopyWith<$Res> {
  _$UpdateRolePermissionsRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of UpdateRolePermissionsRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? permissions = null,
  }) {
    return _then(_value.copyWith(
      permissions: null == permissions
          ? _value.permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$UpdateRolePermissionsRequestImplCopyWith<$Res>
    implements $UpdateRolePermissionsRequestCopyWith<$Res> {
  factory _$$UpdateRolePermissionsRequestImplCopyWith(
          _$UpdateRolePermissionsRequestImpl value,
          $Res Function(_$UpdateRolePermissionsRequestImpl) then) =
      __$$UpdateRolePermissionsRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({List<String> permissions});
}

/// @nodoc
class __$$UpdateRolePermissionsRequestImplCopyWithImpl<$Res>
    extends _$UpdateRolePermissionsRequestCopyWithImpl<$Res,
        _$UpdateRolePermissionsRequestImpl>
    implements _$$UpdateRolePermissionsRequestImplCopyWith<$Res> {
  __$$UpdateRolePermissionsRequestImplCopyWithImpl(
      _$UpdateRolePermissionsRequestImpl _value,
      $Res Function(_$UpdateRolePermissionsRequestImpl) _then)
      : super(_value, _then);

  /// Create a copy of UpdateRolePermissionsRequest
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? permissions = null,
  }) {
    return _then(_$UpdateRolePermissionsRequestImpl(
      permissions: null == permissions
          ? _value._permissions
          : permissions // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$UpdateRolePermissionsRequestImpl
    implements _UpdateRolePermissionsRequest {
  const _$UpdateRolePermissionsRequestImpl(
      {required final List<String> permissions})
      : _permissions = permissions;

  factory _$UpdateRolePermissionsRequestImpl.fromJson(
          Map<String, dynamic> json) =>
      _$$UpdateRolePermissionsRequestImplFromJson(json);

  final List<String> _permissions;
  @override
  List<String> get permissions {
    if (_permissions is EqualUnmodifiableListView) return _permissions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_permissions);
  }

  @override
  String toString() {
    return 'UpdateRolePermissionsRequest(permissions: $permissions)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UpdateRolePermissionsRequestImpl &&
            const DeepCollectionEquality()
                .equals(other._permissions, _permissions));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType, const DeepCollectionEquality().hash(_permissions));

  /// Create a copy of UpdateRolePermissionsRequest
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UpdateRolePermissionsRequestImplCopyWith<
          _$UpdateRolePermissionsRequestImpl>
      get copyWith => __$$UpdateRolePermissionsRequestImplCopyWithImpl<
          _$UpdateRolePermissionsRequestImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$UpdateRolePermissionsRequestImplToJson(
      this,
    );
  }
}

abstract class _UpdateRolePermissionsRequest
    implements UpdateRolePermissionsRequest {
  const factory _UpdateRolePermissionsRequest(
          {required final List<String> permissions}) =
      _$UpdateRolePermissionsRequestImpl;

  factory _UpdateRolePermissionsRequest.fromJson(Map<String, dynamic> json) =
      _$UpdateRolePermissionsRequestImpl.fromJson;

  @override
  List<String> get permissions;

  /// Create a copy of UpdateRolePermissionsRequest
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UpdateRolePermissionsRequestImplCopyWith<
          _$UpdateRolePermissionsRequestImpl>
      get copyWith => throw _privateConstructorUsedError;
}
