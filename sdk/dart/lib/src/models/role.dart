import 'package:freezed_annotation/freezed_annotation.dart';

part 'role.freezed.dart';
part 'role.g.dart';

@freezed
class ApplicationRole with _$ApplicationRole {
  const factory ApplicationRole({
    required String id,
    required String name,
    @JsonKey(name: 'is_system') required bool isSystem,
    required List<String> permissions,
  }) = _ApplicationRole;

  factory ApplicationRole.fromJson(Map<String, dynamic> json) =>
      _$ApplicationRoleFromJson(json);
}

@freezed
class SubjectRoles with _$SubjectRoles {
  const factory SubjectRoles({
    @JsonKey(name: 'subject_id') required String subjectId,
    @JsonKey(name: 'role_ids') required List<String> roleIds,
  }) = _SubjectRoles;

  factory SubjectRoles.fromJson(Map<String, dynamic> json) =>
      _$SubjectRolesFromJson(json);
}

@freezed
class RolePermissions with _$RolePermissions {
  const factory RolePermissions({
    required List<String> permissions,
  }) = _RolePermissions;

  factory RolePermissions.fromJson(Map<String, dynamic> json) =>
      _$RolePermissionsFromJson(json);
}

@freezed
class CreateRoleRequest with _$CreateRoleRequest {
  const factory CreateRoleRequest({
    required String name,
    required List<String> permissions,
  }) = _CreateRoleRequest;

  factory CreateRoleRequest.fromJson(Map<String, dynamic> json) =>
      _$CreateRoleRequestFromJson(json);
}

@freezed
class UpdateRoleRequest with _$UpdateRoleRequest {
  const factory UpdateRoleRequest({
    String? name,
  }) = _UpdateRoleRequest;

  factory UpdateRoleRequest.fromJson(Map<String, dynamic> json) =>
      _$UpdateRoleRequestFromJson(json);
}

@freezed
class UpdateRolePermissionsRequest with _$UpdateRolePermissionsRequest {
  const factory UpdateRolePermissionsRequest({
    required List<String> permissions,
  }) = _UpdateRolePermissionsRequest;

  factory UpdateRolePermissionsRequest.fromJson(Map<String, dynamic> json) =>
      _$UpdateRolePermissionsRequestFromJson(json);
}
