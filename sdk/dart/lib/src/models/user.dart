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
class PatchUserRequest with _$PatchUserRequest {
  const factory PatchUserRequest({
    String? nickname,
  }) = _PatchUserRequest;

  factory PatchUserRequest.fromJson(Map<String, dynamic> json) =>
      _$PatchUserRequestFromJson(json);
}

@freezed
class UpdatePasswordRequest with _$UpdatePasswordRequest {
  const factory UpdatePasswordRequest({required String password}) =
      _UpdatePasswordRequest;

  factory UpdatePasswordRequest.fromJson(Map<String, dynamic> json) =>
      _$UpdatePasswordRequestFromJson(json);
}

/// Query parameters for `GET .../users/search`.
///
/// At least one of [byNickname], [byEmail], [byPhone], or [byId] must be set.
/// Search terms must not contain LIKE wildcards (`%`, `_`, `\`).
@freezed
class SearchApplicationUsersQuery with _$SearchApplicationUsersQuery {
  const factory SearchApplicationUsersQuery({
    @Default(1) int page,
    @JsonKey(name: 'per_page') @Default(30) int perPage,
    @JsonKey(name: 'sort_order') String? sortOrder,
    @JsonKey(name: 'by_nickname') String? byNickname,
    @JsonKey(name: 'by_email') String? byEmail,
    @JsonKey(name: 'by_phone') String? byPhone,
    @JsonKey(name: 'by_id') String? byId,
  }) = _SearchApplicationUsersQuery;

  factory SearchApplicationUsersQuery.fromJson(Map<String, dynamic> json) =>
      _$SearchApplicationUsersQueryFromJson(json);
}

/// @nodoc
@Deprecated('Use SearchApplicationUsersQuery')
typedef UserSearchQuery = SearchApplicationUsersQuery;
