import 'package:freezed_annotation/freezed_annotation.dart';

part 'application.freezed.dart';
part 'application.g.dart';

@freezed
class Application with _$Application {
  const factory Application({
    required String id,
    String? comment,
    @JsonKey(name: 'tenant_id') required String tenantId,
  }) = _Application;

  factory Application.fromJson(Map<String, dynamic> json) =>
      _$ApplicationFromJson(json);
}

@freezed
class ApplicationDetail with _$ApplicationDetail {
  const factory ApplicationDetail({
    required String id,
    String? comment,
    @JsonKey(name: 'tenant_id') required String tenantId,
  }) = _ApplicationDetail;

  factory ApplicationDetail.fromJson(Map<String, dynamic> json) =>
      _$ApplicationDetailFromJson(json);
}

@freezed
class CreateApplicationRequest with _$CreateApplicationRequest {
  const factory CreateApplicationRequest({String? comment}) =
      _CreateApplicationRequest;

  factory CreateApplicationRequest.fromJson(Map<String, dynamic> json) =>
      _$CreateApplicationRequestFromJson(json);
}

@freezed
class UpdateApplicationRequest with _$UpdateApplicationRequest {
  const factory UpdateApplicationRequest({String? comment}) =
      _UpdateApplicationRequest;

  factory UpdateApplicationRequest.fromJson(Map<String, dynamic> json) =>
      _$UpdateApplicationRequestFromJson(json);
}

@freezed
class CreateApplicationResponse with _$CreateApplicationResponse {
  const factory CreateApplicationResponse({
    @JsonKey(name: 'application_id') required String applicationId,
    @JsonKey(name: 'tenant_id') required String tenantId,
    String? comment,
  }) = _CreateApplicationResponse;

  factory CreateApplicationResponse.fromJson(Map<String, dynamic> json) =>
      _$CreateApplicationResponseFromJson(json);
}
