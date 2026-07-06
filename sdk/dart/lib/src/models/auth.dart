import 'package:freezed_annotation/freezed_annotation.dart';

part 'auth.freezed.dart';
part 'auth.g.dart';

@freezed
class SigninRequest with _$SigninRequest {
  const factory SigninRequest({
    required String name,
    required String password,
  }) = _SigninRequest;

  factory SigninRequest.fromJson(Map<String, dynamic> json) =>
      _$SigninRequestFromJson(json);
}

@freezed
class SigninResponse with _$SigninResponse {
  const factory SigninResponse({
    required String jwt,
  }) = _SigninResponse;

  factory SigninResponse.fromJson(Map<String, dynamic> json) =>
      _$SigninResponseFromJson(json);
}

@freezed
class RefreshTokenResponse with _$RefreshTokenResponse {
  const factory RefreshTokenResponse({
    required String jwt,
  }) = _RefreshTokenResponse;

  factory RefreshTokenResponse.fromJson(Map<String, dynamic> json) =>
      _$RefreshTokenResponseFromJson(json);
}

enum TokenDispatchMethod {
  @JsonValue('json')
  json,
  @JsonValue('cookie')
  cookie,
  @JsonValue('both')
  both,
}
