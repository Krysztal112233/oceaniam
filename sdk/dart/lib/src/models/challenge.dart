import 'package:freezed_annotation/freezed_annotation.dart';

part 'challenge.freezed.dart';
part 'challenge.g.dart';

@freezed
class ApplicationChallenge with _$ApplicationChallenge {
  const factory ApplicationChallenge({
    required String id,
    @JsonKey(name: 'factor_type') required String factorType,
    required String purpose,
    required String status,
    @JsonKey(name: 'expires_at') required String expiresAt,
  }) = _ApplicationChallenge;

  factory ApplicationChallenge.fromJson(Map<String, dynamic> json) =>
      _$ApplicationChallengeFromJson(json);
}

@freezed
class SigninChallenge with _$SigninChallenge {
  const factory SigninChallenge({
    @JsonKey(name: 'challenge_id') required String challengeId,
    @JsonKey(name: 'factor_type') required String factorType,
    @JsonKey(name: 'expires_at') required String expiresAt,
  }) = _SigninChallenge;

  factory SigninChallenge.fromJson(Map<String, dynamic> json) =>
      _$SigninChallengeFromJson(json);
}
