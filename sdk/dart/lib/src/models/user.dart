import 'package:freezed_annotation/freezed_annotation.dart';

part 'user.freezed.dart';
part 'user.g.dart';

@freezed
class ApplicationUser with _$ApplicationUser {
  const factory ApplicationUser({
    required String id,
    String? email,
    String? phone,
    required String nickname,
  }) = _ApplicationUser;

  factory ApplicationUser.fromJson(Map<String, dynamic> json) =>
      _$ApplicationUserFromJson(json);
}

@freezed
class CreateUserRequest with _$CreateUserRequest {
  const factory CreateUserRequest({
    required String nickname,
    required String password,
    String? email,
    String? phone,
  }) = _CreateUserRequest;

  factory CreateUserRequest.fromJson(Map<String, dynamic> json) =>
      _$CreateUserRequestFromJson(json);
}

@freezed
class UpdatePasswordRequest with _$UpdatePasswordRequest {
  const factory UpdatePasswordRequest({
    required String password,
  }) = _UpdatePasswordRequest;

  factory UpdatePasswordRequest.fromJson(Map<String, dynamic> json) =>
      _$UpdatePasswordRequestFromJson(json);
}

@freezed
class UserSearchQuery with _$UserSearchQuery {
  const factory UserSearchQuery({
    String? query,
    @JsonKey(name: 'search_by') String? searchBy,
  }) = _UserSearchQuery;

  factory UserSearchQuery.fromJson(Map<String, dynamic> json) =>
      _$UserSearchQueryFromJson(json);
}
