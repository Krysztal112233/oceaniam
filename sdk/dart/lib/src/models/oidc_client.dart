// ignore_for_file: invalid_annotation_target

import 'package:freezed_annotation/freezed_annotation.dart';

part 'oidc_client.freezed.dart';
part 'oidc_client.g.dart';

@freezed
class OidcClient with _$OidcClient {
  const factory OidcClient({
    @JsonKey(name: 'client_id') required String clientId,
    @JsonKey(name: 'application_id') required String applicationId,
    required String name,
    @JsonKey(name: 'client_type') required String clientType,
    @JsonKey(name: 'application_type') required String applicationType,
    @JsonKey(name: 'redirect_uris') required List<String> redirectUris,
    @JsonKey(name: 'created_at') required String createdAt,
  }) = _OidcClient;

  factory OidcClient.fromJson(Map<String, dynamic> json) =>
      _$OidcClientFromJson(json);
}

@freezed
class CreateOidcClientRequest with _$CreateOidcClientRequest {
  const factory CreateOidcClientRequest({
    required String name,
    @JsonKey(name: 'redirect_uris') required List<String> redirectUris,
  }) = _CreateOidcClientRequest;

  factory CreateOidcClientRequest.fromJson(Map<String, dynamic> json) =>
      _$CreateOidcClientRequestFromJson(json);
}

@freezed
class PatchOidcClientRequest with _$PatchOidcClientRequest {
  @JsonSerializable(includeIfNull: false)
  const factory PatchOidcClientRequest({
    String? name,
    @JsonKey(name: 'redirect_uris') List<String>? redirectUris,
  }) = _PatchOidcClientRequest;

  factory PatchOidcClientRequest.fromJson(Map<String, dynamic> json) =>
      _$PatchOidcClientRequestFromJson(_rejectNullPatchFields(json));
}

Map<String, dynamic> _rejectNullPatchFields(Map<String, dynamic> json) {
  for (final field in ['name', 'redirect_uris']) {
    if (json.containsKey(field) && json[field] == null) {
      throw FormatException('$field must not be null');
    }
  }
  return json;
}
