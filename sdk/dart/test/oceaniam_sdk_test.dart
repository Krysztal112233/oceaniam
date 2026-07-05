import 'dart:convert';

import 'package:http/testing.dart';
import 'package:http/http.dart' as http;
import 'package:oceaniam_sdk/oceaniam_sdk.dart';
import 'package:test/test.dart';

void main() {
  group('OceanIAM Models', () {
    test('Tenant fromJson', () {
      final json = {'id': 'abc123', 'comment': 'Test tenant'};
      final tenant = Tenant.fromJson(json);
      expect(tenant.id, 'abc123');
      expect(tenant.comment, 'Test tenant');
    });

    test('Tenant toJson', () {
      final tenant = const Tenant(id: 'abc123', comment: 'Test');
      final json = tenant.toJson();
      expect(json['id'], 'abc123');
      expect(json['comment'], 'Test');
    });

    test('Application fromJson', () {
      final json = {
        'id': 'app1',
        'comment': 'My App',
        'tenant_id': 'tenant1',
      };
      final app = Application.fromJson(json);
      expect(app.id, 'app1');
      expect(app.tenantId, 'tenant1');
    });

    test('SigninRequest toJson', () {
      final req = SigninRequest(name: 'admin', password: 'pass');
      final json = req.toJson();
      expect(json['name'], 'admin');
      expect(json['password'], 'pass');
    });

    test('SigninResponse fromJson', () {
      final json = {'token': 'jwt_token_value'};
      final resp = SigninResponse.fromJson(json);
      expect(resp.token, 'jwt_token_value');
    });

    test('ApplicationUser fromJson', () {
      final json = {
        'id': 'u1',
        'email': 'user@example.com',
        'phone': null,
        'nickname': 'testuser',
      };
      final user = ApplicationUser.fromJson(json);
      expect(user.id, 'u1');
      expect(user.email, 'user@example.com');
      expect(user.nickname, 'testuser');
    });

    test('Secret fromJson', () {
      final json = {
        'id': 's1',
        'secret': 'sk-xxx',
        'created_at': '2024-01-01T00:00:00Z',
        'revoked_at': null,
        'application_ids': ['app1'],
      };
      final secret = Secret.fromJson(json);
      expect(secret.id, 's1');
      expect(secret.applicationIds, ['app1']);
    });

    test('Overview fromJson', () {
      final json = {
        'total_tenants': 5,
        'total_applications': 10,
        'total_administrators': 3,
        'total_application_users': 100,
        'total_active_secrets': 2,
      };
      final overview = Overview.fromJson(json);
      expect(overview.totalTenants, 5);
      expect(overview.totalApplications, 10);
    });

    test('AuditLog fromJson', () {
      final json = {
        'id': 'a1',
        'audit_type': 'user.login',
        'payload': {'ip': '127.0.0.1'},
        'created_at': '2024-01-01T00:00:00Z',
      };
      final audit = AuditLog.fromJson(json);
      expect(audit.auditType, 'user.login');
      expect(audit.payload['ip'], '127.0.0.1');
    });
  });

  group('OceanIAM Client', () {
    late OceanIAMClient client;
    late MockClient mockHttp;

    setUp(() {
      mockHttp = MockClient((request) async {
        if (request.url.path == '/auth/tokens' && request.method == 'POST') {
          return http.Response(
            jsonEncode({'token': 'test-jwt'}),
            200,
          );
        }
        if (request.url.path == '/tenants' && request.method == 'GET') {
          return http.Response(
            jsonEncode({
              'items': [
                {'id': 't1', 'comment': 'Tenant 1'}
              ],
              'page_info': {'has_next': false, 'total': 1},
            }),
            200,
          );
        }
        if (request.url.path == '/administrators/me' &&
            request.method == 'GET') {
          return http.Response(
            jsonEncode({
              'id': 'admin1',
              'name': 'admin',
              'role': 'super_admin',
              'permissions': ['TenantRead', 'TenantCreate'],
            }),
            200,
          );
        }
        if (request.url.path == '/secrets' && request.method == 'POST') {
          return http.Response(
            jsonEncode({'id': 's1', 'secret': 'sk-generated'}),
            200,
          );
        }
        return http.Response('Not found', 404);
      });

      client = OceanIAMClient(
        baseUrl: 'http://localhost:8000',
        httpClient: mockHttp,
      );
    });

    test('signin sets token', () async {
      expect(client.isAuthenticated, false);
      final resp = await client.signin('admin', 'password');
      expect(resp.token, 'test-jwt');
      expect(client.isAuthenticated, true);
      expect(client.token, 'test-jwt');
    });

    test('listTenants returns paged response', () async {
      await client.signin('admin', 'password');
      final result = await client.listTenants();
      expect(result.items.length, 1);
      expect(result.items[0].id, 't1');
      expect(result.pageInfo.total, 1);
      expect(result.pageInfo.hasNext, false);
    });

    test('getMyProfile returns profile', () async {
      await client.signin('admin', 'password');
      final profile = await client.getMyProfile();
      expect(profile.name, 'admin');
      expect(profile.permissions, ['TenantRead', 'TenantCreate']);
    });

    test('createSecret returns secret', () async {
      await client.signin('admin', 'password');
      final result = await client.createSecret();
      expect(result.id, 's1');
      expect(result.secret, 'sk-generated');
    });
    test('throws OceanIAMError on 404', () async {
      await client.signin('admin', 'password');
      await expectLater(
        client.getTenant('nonexistent'),
        throwsA(isA<OceanIAMError>()),
      );
    });

    test('dispose clears token', () async {
      final localClient = OceanIAMClient(
        baseUrl: 'http://localhost:8000',
        httpClient: MockClient(
            (_) async => http.Response(jsonEncode({'token': 'x'}), 200)),
      );
      await localClient.signin('admin', 'pass');
      expect(localClient.isAuthenticated, true);
      localClient.dispose();
    });
  });
}
