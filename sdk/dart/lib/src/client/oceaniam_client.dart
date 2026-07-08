import 'dart:convert';

import 'package:http/http.dart' as http;

import '../models/administrator.dart';
import '../models/application.dart';
import '../models/audit.dart';
import '../models/auth.dart';
import '../models/configuration.dart';
import '../models/key.dart';
import '../models/pagination.dart';
import '../models/role.dart';
import '../models/secret.dart';
import '../models/statistics.dart';
import '../models/tenant.dart';
import '../models/user.dart';

class OceanIAMError implements Exception {
  final int statusCode;
  final String message;
  final Map<String, dynamic>? details;

  const OceanIAMError({
    required this.statusCode,
    required this.message,
    this.details,
  });

  @override
  String toString() => 'OceanIAMError($statusCode): $message';
}

class OceanIAMClient {
  final String baseUrl;
  final http.Client _httpClient;
  String? _jwt;

  OceanIAMClient({required this.baseUrl, http.Client? httpClient})
      : _httpClient = httpClient ?? http.Client();

  bool get isAuthenticated => _jwt != null;

  void setToken(String? token) {
    _jwt = token;
  }

  String? get token => _jwt;

  Map<String, String> get _headers {
    final headers = <String, String>{'Content-Type': 'application/json'};
    if (_jwt != null) {
      headers['Authorization'] = 'Bearer $_jwt';
    }
    return headers;
  }

  Future<Map<String, dynamic>> _request(
    String method,
    String path, {
    Map<String, dynamic>? body,
    Map<String, String>? extraHeaders,
  }) async {
    final uri = Uri.parse('$baseUrl$path');
    final headers = {..._headers, if (extraHeaders != null) ...extraHeaders};

    late http.Response response;
    switch (method) {
      case 'GET':
        response = await _httpClient.get(uri, headers: headers);
        break;
      case 'POST':
        response = await _httpClient.post(
          uri,
          headers: headers,
          body: body != null ? jsonEncode(body) : null,
        );
        break;
      case 'PATCH':
        response = await _httpClient.patch(
          uri,
          headers: headers,
          body: body != null ? jsonEncode(body) : null,
        );
        break;
      case 'PUT':
        response = await _httpClient.put(
          uri,
          headers: headers,
          body: body != null ? jsonEncode(body) : null,
        );
        break;
      case 'DELETE':
        response = await _httpClient.delete(uri, headers: headers);
        break;
      default:
        throw ArgumentError('Unsupported HTTP method: $method');
    }

    if (response.statusCode >= 200 && response.statusCode < 300) {
      if (response.body.isEmpty) return {};
      return jsonDecode(response.body) as Map<String, dynamic>;
    }

    Map<String, dynamic>? errorBody;
    try {
      if (response.body.isNotEmpty) {
        errorBody = jsonDecode(response.body) as Map<String, dynamic>;
      }
    } catch (_) {}

    throw OceanIAMError(
      statusCode: response.statusCode,
      message: errorBody?['error']?.toString() ??
          response.reasonPhrase ??
          'Unknown error',
      details: errorBody,
    );
  }

  Future<List<Map<String, dynamic>>> _requestList(
    String method,
    String path, {
    Map<String, dynamic>? body,
  }) async {
    final uri = Uri.parse('$baseUrl$path');
    final headers = _headers;

    late http.Response response;
    switch (method) {
      case 'GET':
        response = await _httpClient.get(uri, headers: headers);
        break;
      case 'POST':
        response = await _httpClient.post(
          uri,
          headers: headers,
          body: body != null ? jsonEncode(body) : null,
        );
        break;
      default:
        throw ArgumentError('Unsupported HTTP method for list: $method');
    }

    if (response.statusCode >= 200 && response.statusCode < 300) {
      if (response.body.isEmpty) return [];
      final decoded = jsonDecode(response.body);
      return (decoded as List<dynamic>).cast<Map<String, dynamic>>();
    }

    Map<String, dynamic>? errorBody;
    try {
      if (response.body.isNotEmpty) {
        errorBody = jsonDecode(response.body) as Map<String, dynamic>;
      }
    } catch (_) {}

    throw OceanIAMError(
      statusCode: response.statusCode,
      message: errorBody?['error']?.toString() ??
          response.reasonPhrase ??
          'Unknown error',
      details: errorBody,
    );
  }

  Future<void> _requestNoContent(
    String method,
    String path, {
    Map<String, dynamic>? body,
  }) async {
    final uri = Uri.parse('$baseUrl$path');
    final headers = _headers;

    late http.Response response;
    switch (method) {
      case 'DELETE':
        response = await _httpClient.delete(uri, headers: headers);
        break;
      case 'POST':
        response = await _httpClient.post(
          uri,
          headers: headers,
          body: body != null ? jsonEncode(body) : null,
        );
        break;
      case 'PATCH':
        response = await _httpClient.patch(
          uri,
          headers: headers,
          body: body != null ? jsonEncode(body) : null,
        );
        break;
      case 'PUT':
        response = await _httpClient.put(
          uri,
          headers: headers,
          body: body != null ? jsonEncode(body) : null,
        );
        break;
      default:
        throw ArgumentError('Unsupported HTTP method: $method');
    }

    if (response.statusCode >= 200 && response.statusCode < 300) {
      return;
    }

    Map<String, dynamic>? errorBody;
    try {
      if (response.body.isNotEmpty) {
        errorBody = jsonDecode(response.body) as Map<String, dynamic>;
      }
    } catch (_) {}

    throw OceanIAMError(
      statusCode: response.statusCode,
      message: errorBody?['error']?.toString() ??
          response.reasonPhrase ??
          'Unknown error',
      details: errorBody,
    );
  }

  // =========================================================================
  // Auth
  // =========================================================================

  Future<SigninResponse> signin(String name, String password) async {
    final data = await _request(
      'POST',
      '/auth/tokens',
      body: SigninRequest(name: name, password: password).toJson(),
      extraHeaders: {'X-OceanIAM-Token-Dispatch': 'json'},
    );
    final response = SigninResponse.fromJson(data);
    _jwt = response.jwt;
    return response;
  }

  Future<void> signout() async {
    try {
      await _requestNoContent('DELETE', '/auth/tokens');
    } finally {
      _jwt = null;
    }
  }

  Future<RefreshTokenResponse> refreshToken() async {
    final data = await _request('POST', '/auth/tokens/refresh');
    final response = RefreshTokenResponse.fromJson(data);
    _jwt = response.jwt;
    return response;
  }

  // =========================================================================
  // Tenants
  // =========================================================================

  Future<PagedResponse<Tenant>> listTenants({
    int page = 1,
    int pageSize = 20,
  }) async {
    final data = await _request(
      'GET',
      '/tenants?page=$page&per_page=$pageSize',
    );
    return PagedResponse<Tenant>(
      items: (data['items'] as List<dynamic>)
          .map((e) => Tenant.fromJson(e as Map<String, dynamic>))
          .toList(),
      pageInfo: PageInfo.fromJson(data['page_info'] as Map<String, dynamic>),
    );
  }

  Future<Tenant> getTenant(String tenantId) async {
    final data = await _request('GET', '/tenants/$tenantId');
    return Tenant.fromJson(data);
  }

  Future<Tenant> createTenant({String? comment}) async {
    final body = CreateTenantRequest(comment: comment);
    final data = await _request('POST', '/tenants', body: body.toJson());
    return Tenant.fromJson(data);
  }

  Future<void> updateTenant(String tenantId, {String? comment}) async {
    final body = UpdateTenantRequest(comment: comment);
    await _requestNoContent('PATCH', '/tenants/$tenantId', body: body.toJson());
  }

  Future<void> deleteTenant(String tenantId) async {
    await _requestNoContent('DELETE', '/tenants/$tenantId');
  }

  Future<List<ApplicationUser>> listTenantUsers(String tenantId) async {
    final data = await _requestList('GET', '/tenants/$tenantId/users');
    return data.map((e) => ApplicationUser.fromJson(e)).toList();
  }

  // =========================================================================
  // Administrators
  // =========================================================================

  Future<PagedResponse<Administrator>> listAdministrators({
    int page = 1,
    int pageSize = 20,
  }) async {
    final data = await _request(
      'GET',
      '/administrators?page=$page&per_page=$pageSize',
    );
    return PagedResponse<Administrator>(
      items: (data['items'] as List<dynamic>)
          .map((e) => Administrator.fromJson(e as Map<String, dynamic>))
          .toList(),
      pageInfo: PageInfo.fromJson(data['page_info'] as Map<String, dynamic>),
    );
  }

  Future<AdministratorProfile> getMyProfile() async {
    final data = await _request('GET', '/administrators/me');
    return AdministratorProfile.fromJson(data);
  }

  Future<CreateAdministratorResponse> createAdministrator(
    String name,
    String password,
  ) async {
    final body = CreateAdministratorRequest(name: name, password: password);
    final data = await _request('POST', '/administrators', body: body.toJson());
    return CreateAdministratorResponse.fromJson(data);
  }

  Future<void> updateAdministrator(
    String adminId, {
    String? name,
    String? password,
  }) async {
    final body = UpdateAdministratorRequest(name: name, password: password);
    await _requestNoContent(
      'PATCH',
      '/administrators/$adminId',
      body: body.toJson(),
    );
  }

  // =========================================================================
  // Applications
  // =========================================================================

  Future<PagedResponse<Application>> listApplications(
    String tenantId, {
    int page = 1,
    int pageSize = 20,
  }) async {
    final data = await _request(
      'GET',
      '/tenants/$tenantId/applications?page=$page&per_page=$pageSize',
    );
    return PagedResponse<Application>(
      items: (data['items'] as List<dynamic>)
          .map((e) => Application.fromJson(e as Map<String, dynamic>))
          .toList(),
      pageInfo: PageInfo.fromJson(data['page_info'] as Map<String, dynamic>),
    );
  }

  Future<ApplicationDetail> getApplication(
    String tenantId,
    String applicationId,
  ) async {
    final data = await _request(
      'GET',
      '/tenants/$tenantId/applications/$applicationId',
    );
    return ApplicationDetail.fromJson(data);
  }

  Future<CreateApplicationResponse> createApplication(
    String tenantId, {
    String? comment,
  }) async {
    final body = CreateApplicationRequest(comment: comment);
    final data = await _request(
      'POST',
      '/tenants/$tenantId/applications',
      body: body.toJson(),
    );
    return CreateApplicationResponse.fromJson(data);
  }

  Future<void> updateApplication(
    String tenantId,
    String applicationId, {
    String? comment,
  }) async {
    final body = UpdateApplicationRequest(comment: comment);
    await _requestNoContent(
      'PATCH',
      '/tenants/$tenantId/applications/$applicationId',
      body: body.toJson(),
    );
  }

  Future<void> deleteApplication(String tenantId, String applicationId) async {
    await _requestNoContent(
      'DELETE',
      '/tenants/$tenantId/applications/$applicationId',
    );
  }

  Future<ApplicationConfiguration> getApplicationConfiguration(
    String tenantId,
    String applicationId,
  ) async {
    final data = await _request(
      'GET',
      '/tenants/$tenantId/applications/$applicationId/configuration',
    );
    return ApplicationConfiguration.fromJson(data);
  }

  Future<void> updateApplicationConfiguration(
    String tenantId,
    String applicationId,
    ApplicationConfiguration config,
  ) async {
    await _requestNoContent(
      'PATCH',
      '/tenants/$tenantId/applications/$applicationId/configuration',
      body: config.toJson(),
    );
  }

  // =========================================================================
  // Application Users
  // =========================================================================

  Future<PagedResponse<ApplicationUser>> listUsers(
    String tenantId,
    String applicationId, {
    int page = 1,
    int pageSize = 20,
    String? sortOrder,
  }) async {
    var path =
        '/tenants/$tenantId/applications/$applicationId/users?page=$page&per_page=$pageSize';
    if (sortOrder != null) path += '&sort_order=$sortOrder';
    final data = await _request('GET', path);
    return PagedResponse<ApplicationUser>(
      items: (data['items'] as List<dynamic>)
          .map((e) => ApplicationUser.fromJson(e as Map<String, dynamic>))
          .toList(),
      pageInfo: PageInfo.fromJson(data['page_info'] as Map<String, dynamic>),
    );
  }

  Future<List<ApplicationUser>> searchUsers(
    String tenantId,
    String applicationId, {
    String? query,
    String? searchBy,
  }) async {
    var path = '/tenants/$tenantId/applications/$applicationId/users/search';
    final params = <String>[];
    if (query != null) params.add('query=$query');
    if (searchBy != null) params.add('search_by=$searchBy');
    if (params.isNotEmpty) path += '?${params.join('&')}';
    final data = await _requestList('GET', path);
    return data.map((e) => ApplicationUser.fromJson(e)).toList();
  }

  Future<ApplicationUser> createUser(
    String tenantId,
    String applicationId,
    CreateUserRequest request,
  ) async {
    final data = await _request(
      'POST',
      '/tenants/$tenantId/applications/$applicationId/users',
      body: request.toJson(),
    );
    return ApplicationUser.fromJson(data);
  }

  Future<ApplicationUser> getUser(
    String tenantId,
    String applicationId,
    String userId,
  ) async {
    final data = await _request(
      'GET',
      '/tenants/$tenantId/applications/$applicationId/users/$userId',
    );
    return ApplicationUser.fromJson(data);
  }

  Future<void> updateUserPassword(
    String tenantId,
    String applicationId,
    String userId,
    String password,
  ) async {
    final body = UpdatePasswordRequest(password: password);
    await _requestNoContent(
      'PATCH',
      '/tenants/$tenantId/applications/$applicationId/users/$userId/credentials',
      body: body.toJson(),
    );
  }

  // =========================================================================
  // Secrets
  // =========================================================================

  Future<CreateSecretResponse> createSecret() async {
    final data = await _request('POST', '/secrets');
    return CreateSecretResponse.fromJson(data);
  }

  Future<PagedResponse<Secret>> listSecrets({
    int page = 1,
    int pageSize = 20,
  }) async {
    final data = await _request(
      'GET',
      '/secrets?page=$page&per_page=$pageSize',
    );
    return PagedResponse<Secret>(
      items: (data['items'] as List<dynamic>)
          .map((e) => Secret.fromJson(e as Map<String, dynamic>))
          .toList(),
      pageInfo: PageInfo.fromJson(data['page_info'] as Map<String, dynamic>),
    );
  }

  Future<Secret> getSecret(String secretId) async {
    final data = await _request('GET', '/secrets/$secretId');
    return Secret.fromJson(data);
  }

  Future<void> deleteSecret(String secretId) async {
    await _requestNoContent('DELETE', '/secrets/$secretId');
  }

  Future<void> bindSecret(String secretId, String applicationId) async {
    final body = BindSecretRequest(applicationId: applicationId);
    await _requestNoContent(
      'POST',
      '/secrets/$secretId/bindings',
      body: body.toJson(),
    );
  }

  Future<void> unbindSecret(String secretId, String applicationId) async {
    await _requestNoContent(
      'DELETE',
      '/secrets/$secretId/bindings/$applicationId',
    );
  }

  Future<List<Secret>> listApplicationSecrets(
    String tenantId,
    String applicationId,
  ) async {
    final data = await _requestList(
      'GET',
      '/tenants/$tenantId/applications/$applicationId/secrets',
    );
    return data.map((e) => Secret.fromJson(e)).toList();
  }

  // =========================================================================
  // Roles
  // =========================================================================

  Future<List<ApplicationRole>> listRoles(
    String tenantId,
    String applicationId,
  ) async {
    final data = await _requestList(
      'GET',
      '/tenants/$tenantId/applications/$applicationId/roles',
    );
    return data.map((e) => ApplicationRole.fromJson(e)).toList();
  }

  Future<ApplicationRole> createRole(
    String tenantId,
    String applicationId,
    CreateRoleRequest request,
  ) async {
    final data = await _request(
      'POST',
      '/tenants/$tenantId/applications/$applicationId/roles',
      body: request.toJson(),
    );
    return ApplicationRole.fromJson(data);
  }

  Future<ApplicationRole> getRole(
    String tenantId,
    String applicationId,
    String roleId,
  ) async {
    final data = await _request(
      'GET',
      '/tenants/$tenantId/applications/$applicationId/roles/$roleId',
    );
    return ApplicationRole.fromJson(data);
  }

  Future<void> updateRole(
    String tenantId,
    String applicationId,
    String roleId,
    UpdateRoleRequest request,
  ) async {
    await _requestNoContent(
      'PATCH',
      '/tenants/$tenantId/applications/$applicationId/roles/$roleId',
      body: request.toJson(),
    );
  }

  Future<void> deleteRole(
    String tenantId,
    String applicationId,
    String roleId,
  ) async {
    await _requestNoContent(
      'DELETE',
      '/tenants/$tenantId/applications/$applicationId/roles/$roleId',
    );
  }

  Future<RolePermissions> getRolePermissions(
    String tenantId,
    String applicationId,
    String roleId,
  ) async {
    final data = await _request(
      'GET',
      '/tenants/$tenantId/applications/$applicationId/roles/$roleId/permissions',
    );
    return RolePermissions.fromJson(data);
  }

  Future<void> updateRolePermissions(
    String tenantId,
    String applicationId,
    String roleId,
    UpdateRolePermissionsRequest request,
  ) async {
    await _requestNoContent(
      'PUT',
      '/tenants/$tenantId/applications/$applicationId/roles/$roleId/permissions',
      body: request.toJson(),
    );
  }

  Future<SubjectRoles> getSubjectRoles(
    String tenantId,
    String applicationId,
    String subjectId,
  ) async {
    final data = await _request(
      'GET',
      '/tenants/$tenantId/applications/$applicationId/roles/subjects/$subjectId/roles',
    );
    return SubjectRoles.fromJson(data);
  }

  Future<void> assignRole(
    String tenantId,
    String applicationId,
    String subjectId,
    String roleId,
  ) async {
    await _requestNoContent(
      'POST',
      '/tenants/$tenantId/applications/$applicationId/roles/subjects/$subjectId/roles',
      body: {'role_id': roleId},
    );
  }

  Future<void> unassignRole(
    String tenantId,
    String applicationId,
    String subjectId,
    String roleId,
  ) async {
    await _requestNoContent(
      'DELETE',
      '/tenants/$tenantId/applications/$applicationId/roles/subjects/$subjectId/roles/$roleId',
    );
  }

  // =========================================================================
  // Keys
  // =========================================================================

  Future<List<ApplicationKey>> listKeys(String tenantId) async {
    final data = await _requestList('GET', '/tenants/$tenantId/keys');
    return data.map((e) => ApplicationKey.fromJson(e)).toList();
  }

  Future<ApplicationKey> rotateKey(String tenantId) async {
    final data = await _request('POST', '/tenants/$tenantId/keys');
    return ApplicationKey.fromJson(data);
  }

  Future<void> revokeKey(String tenantId, String keyId) async {
    await _requestNoContent('DELETE', '/tenants/$tenantId/keys/$keyId');
  }

  // =========================================================================
  // Statistics
  // =========================================================================

  Future<Overview> getOverview() async {
    final data = await _request('GET', '/statistics');
    return Overview.fromJson(data);
  }

  Future<PlatformTrends> getPlatformTrends({
    String? granularity,
    int? range,
  }) async {
    var path = '/statistics/trends';
    final params = <String>[];
    if (granularity != null) params.add('granularity=$granularity');
    if (range != null) params.add('range=$range');
    if (params.isNotEmpty) path += '?${params.join('&')}';
    final data = await _request('GET', path);
    return PlatformTrends.fromJson(data);
  }

  Future<ApplicationStatistics> getApplicationStatistics(
    String tenantId,
    String applicationId,
  ) async {
    final data = await _request(
      'GET',
      '/tenants/$tenantId/applications/$applicationId/statistics',
    );
    return ApplicationStatistics.fromJson(data);
  }

  Future<ApplicationTrends> getApplicationTrends(
    String tenantId,
    String applicationId, {
    String? granularity,
    int? range,
  }) async {
    var path =
        '/tenants/$tenantId/applications/$applicationId/statistics/trends';
    final params = <String>[];
    if (granularity != null) params.add('granularity=$granularity');
    if (range != null) params.add('range=$range');
    if (params.isNotEmpty) path += '?${params.join('&')}';
    final data = await _request('GET', path);
    return ApplicationTrends.fromJson(data);
  }

  // =========================================================================
  // Audits
  // =========================================================================

  Future<PagedResponse<AuditLog>> listAudits({
    int page = 1,
    int pageSize = 20,
    String? auditType,
  }) async {
    var path = '/audits?page=$page&per_page=$pageSize';
    if (auditType != null) path += '&audit_type=$auditType';
    final data = await _request('GET', path);
    return PagedResponse<AuditLog>(
      items: (data['items'] as List<dynamic>)
          .map((e) => AuditLog.fromJson(e as Map<String, dynamic>))
          .toList(),
      pageInfo: PageInfo.fromJson(data['page_info'] as Map<String, dynamic>),
    );
  }

  // =========================================================================
  // Application-scoped Audits
  // =========================================================================

  Future<PagedResponse<AuditLog>> listApplicationAudits(
    String tenantId,
    String applicationId, {
    int page = 1,
    int pageSize = 20,
    String? auditType,
  }) async {
    var path =
        '/tenants/$tenantId/applications/$applicationId/audits?page=$page&per_page=$pageSize';
    if (auditType != null) path += '&audit_type=$auditType';
    final data = await _request('GET', path);
    return PagedResponse<AuditLog>(
      items: (data['items'] as List<dynamic>)
          .map((e) => AuditLog.fromJson(e as Map<String, dynamic>))
          .toList(),
      pageInfo: PageInfo.fromJson(data['page_info'] as Map<String, dynamic>),
    );
  }

  void dispose() {
    _httpClient.close();
  }
}
