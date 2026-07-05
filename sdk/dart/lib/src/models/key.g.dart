// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'key.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ApplicationKeyImpl _$$ApplicationKeyImplFromJson(Map<String, dynamic> json) =>
    _$ApplicationKeyImpl(
      keyId: json['key_id'] as String,
      algorithm: json['algorithm'] as String,
      status: json['status'] as String,
      activatedAt: json['activated_at'] as String,
    );

Map<String, dynamic> _$$ApplicationKeyImplToJson(
        _$ApplicationKeyImpl instance) =>
    <String, dynamic>{
      'key_id': instance.keyId,
      'algorithm': instance.algorithm,
      'status': instance.status,
      'activated_at': instance.activatedAt,
    };
