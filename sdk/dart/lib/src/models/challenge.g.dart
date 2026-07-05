// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'challenge.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ApplicationChallengeImpl _$$ApplicationChallengeImplFromJson(
        Map<String, dynamic> json) =>
    _$ApplicationChallengeImpl(
      id: json['id'] as String,
      factorType: json['factor_type'] as String,
      purpose: json['purpose'] as String,
      status: json['status'] as String,
      expiresAt: json['expires_at'] as String,
    );

Map<String, dynamic> _$$ApplicationChallengeImplToJson(
        _$ApplicationChallengeImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'factor_type': instance.factorType,
      'purpose': instance.purpose,
      'status': instance.status,
      'expires_at': instance.expiresAt,
    };

_$SigninChallengeImpl _$$SigninChallengeImplFromJson(
        Map<String, dynamic> json) =>
    _$SigninChallengeImpl(
      challengeId: json['challenge_id'] as String,
      factorType: json['factor_type'] as String,
      expiresAt: json['expires_at'] as String,
    );

Map<String, dynamic> _$$SigninChallengeImplToJson(
        _$SigninChallengeImpl instance) =>
    <String, dynamic>{
      'challenge_id': instance.challengeId,
      'factor_type': instance.factorType,
      'expires_at': instance.expiresAt,
    };
