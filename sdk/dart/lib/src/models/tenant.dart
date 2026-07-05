import 'package:freezed_annotation/freezed_annotation.dart';

part 'tenant.freezed.dart';
part 'tenant.g.dart';

@freezed
class Tenant with _$Tenant {
  const factory Tenant({
    required String id,
    String? comment,
  }) = _Tenant;

  factory Tenant.fromJson(Map<String, dynamic> json) => _$TenantFromJson(json);
}

@freezed
class CreateTenantRequest with _$CreateTenantRequest {
  const factory CreateTenantRequest({
    String? comment,
  }) = _CreateTenantRequest;

  factory CreateTenantRequest.fromJson(Map<String, dynamic> json) =>
      _$CreateTenantRequestFromJson(json);
}

@freezed
class UpdateTenantRequest with _$UpdateTenantRequest {
  const factory UpdateTenantRequest({
    String? comment,
  }) = _UpdateTenantRequest;

  factory UpdateTenantRequest.fromJson(Map<String, dynamic> json) =>
      _$UpdateTenantRequestFromJson(json);
}
