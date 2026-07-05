import 'package:freezed_annotation/freezed_annotation.dart';

part 'key.freezed.dart';
part 'key.g.dart';

@freezed
class ApplicationKey with _$ApplicationKey {
  const factory ApplicationKey({
    @JsonKey(name: 'key_id') required String keyId,
    required String algorithm,
    required String status,
    @JsonKey(name: 'activated_at') required String activatedAt,
  }) = _ApplicationKey;

  factory ApplicationKey.fromJson(Map<String, dynamic> json) =>
      _$ApplicationKeyFromJson(json);
}
