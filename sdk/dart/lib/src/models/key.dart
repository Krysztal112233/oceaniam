import 'package:freezed_annotation/freezed_annotation.dart';

part 'key.freezed.dart';
part 'key.g.dart';

@freezed
class ApplicationKey with _$ApplicationKey {
  const factory ApplicationKey({
    @JsonKey(name: 'key_id') required String keyId,
    required String algorithm,
    required String status,
    @JsonKey(name: 'created_at') required String createdAt,
    @JsonKey(name: 'activated_at') required String activatedAt,
    @JsonKey(name: 'retired_at') required String retiredAt,
    @JsonKey(name: 'expires_at') required String expiresAt,
    @JsonKey(name: 'revoked_at') String? revokedAt,
  }) = _ApplicationKey;

  factory ApplicationKey.fromJson(Map<String, dynamic> json) =>
      _$ApplicationKeyFromJson(json);
}
