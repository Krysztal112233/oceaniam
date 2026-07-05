import 'package:freezed_annotation/freezed_annotation.dart';

part 'secret.freezed.dart';
part 'secret.g.dart';

@freezed
class Secret with _$Secret {
  const factory Secret({
    required String id,
    String? secret,
    @JsonKey(name: 'created_at') required String createdAt,
    @JsonKey(name: 'revoked_at') String? revokedAt,
    @JsonKey(name: 'application_ids') required List<String> applicationIds,
  }) = _Secret;

  factory Secret.fromJson(Map<String, dynamic> json) => _$SecretFromJson(json);
}

@freezed
class CreateSecretResponse with _$CreateSecretResponse {
  const factory CreateSecretResponse({
    required String id,
    required String secret,
  }) = _CreateSecretResponse;

  factory CreateSecretResponse.fromJson(Map<String, dynamic> json) =>
      _$CreateSecretResponseFromJson(json);
}

@freezed
class BindSecretRequest with _$BindSecretRequest {
  const factory BindSecretRequest({
    @JsonKey(name: 'application_id') required String applicationId,
  }) = _BindSecretRequest;

  factory BindSecretRequest.fromJson(Map<String, dynamic> json) =>
      _$BindSecretRequestFromJson(json);
}
