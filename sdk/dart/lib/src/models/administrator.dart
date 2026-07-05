import 'package:freezed_annotation/freezed_annotation.dart';

part 'administrator.freezed.dart';
part 'administrator.g.dart';

@freezed
class Administrator with _$Administrator {
  const factory Administrator({
    required String id,
    required String name,
    String? role,
  }) = _Administrator;

  factory Administrator.fromJson(Map<String, dynamic> json) =>
      _$AdministratorFromJson(json);
}

@freezed
class AdministratorProfile with _$AdministratorProfile {
  const factory AdministratorProfile({
    required String id,
    required String name,
    String? role,
    required List<String> permissions,
  }) = _AdministratorProfile;

  factory AdministratorProfile.fromJson(Map<String, dynamic> json) =>
      _$AdministratorProfileFromJson(json);
}

@freezed
class CreateAdministratorRequest with _$CreateAdministratorRequest {
  const factory CreateAdministratorRequest({
    required String name,
    required String password,
  }) = _CreateAdministratorRequest;

  factory CreateAdministratorRequest.fromJson(Map<String, dynamic> json) =>
      _$CreateAdministratorRequestFromJson(json);
}

@freezed
class CreateAdministratorResponse with _$CreateAdministratorResponse {
  const factory CreateAdministratorResponse({
    required String id,
    required String name,
    required String password,
  }) = _CreateAdministratorResponse;

  factory CreateAdministratorResponse.fromJson(Map<String, dynamic> json) =>
      _$CreateAdministratorResponseFromJson(json);
}

@freezed
class UpdateAdministratorRequest with _$UpdateAdministratorRequest {
  const factory UpdateAdministratorRequest({
    String? name,
    String? password,
  }) = _UpdateAdministratorRequest;

  factory UpdateAdministratorRequest.fromJson(Map<String, dynamic> json) =>
      _$UpdateAdministratorRequestFromJson(json);
}
