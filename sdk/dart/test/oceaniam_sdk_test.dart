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
      final json = {'id': 'app1', 'comment': 'My App', 'tenant_id': 'tenant1'};
      final app = Application.fromJson(json);
      expect(app.id, 'app1');
      expect(app.tenantId, 'tenant1');
    });

    test('OIDC client models use the backend wire field names', () {
      final client = OidcClient.fromJson({
        'client_id': 'client-sqid',
        'application_id': 'app1',
        'name': 'Web client',
        'client_type': 'public',
        'application_type': 'web',
        'redirect_uris': ['https://client.example/callback'],
        'created_at': '2026-09-07T00:00:00Z',
      });
      expect(client.clientId, 'client-sqid');
      expect(client.applicationId, 'app1');
      expect(client.clientType, 'public');
      expect(client.applicationType, 'web');

      const request = CreateOidcClientRequest(
        name: 'Web client',
        redirectUris: ['https://client.example/callback'],
      );
      expect(request.toJson(), {
        'name': 'Web client',
        'redirect_uris': ['https://client.example/callback'],
      });

      const emptyPatch = PatchOidcClientRequest();
      expect(emptyPatch.toJson(), isEmpty);
      const redirectPatch = PatchOidcClientRequest(
        redirectUris: ['https://CLIENT.example/callback/%2Fraw'],
      );
      expect(redirectPatch.toJson(), {
        'redirect_uris': ['https://CLIENT.example/callback/%2Fraw'],
      });
      expect(
        () => PatchOidcClientRequest.fromJson({'name': null}),
        throwsFormatException,
      );
      expect(
        () => PatchOidcClientRequest.fromJson({'redirect_uris': null}),
        throwsFormatException,
      );
    });

    test('SigninRequest toJson', () {
      final req = SigninRequest(name: 'admin', password: 'pass');
      final json = req.toJson();
      expect(json['name'], 'admin');
      expect(json['password'], 'pass');
    });

    test('SigninResponse fromJson', () {
      final json = {'jwt': 'jwt_token_value'};
      final resp = SigninResponse.fromJson(json);
      expect(resp.jwt, 'jwt_token_value');
    });

    test('ApplicationUser fromJson', () {
      final json = {
        'id': 'u1',
        'oidc_sub': '018f3f47-7b2f-7000-8000-000000000001',
        'email': 'user@example.com',
        'phone': null,
        'nickname': 'testuser',
      };
      final user = ApplicationUser.fromJson(json);
      expect(user.id, 'u1');
      expect(user.oidcSub, '018f3f47-7b2f-7000-8000-000000000001');
      expect(user.email, 'user@example.com');
      expect(user.nickname, 'testuser');
      expect(user.toJson()['oidc_sub'], user.oidcSub);
      expect(
        () => ApplicationUser.fromJson({
          'id': 'u1',
          'email': null,
          'phone': null,
          'nickname': 'testuser',
        }),
        throwsA(isA<TypeError>()),
      );
      expect(
        () => ApplicationUser.fromJson({
          'id': 'u1',
          'oidc_sub': null,
          'email': null,
          'phone': null,
          'nickname': 'testuser',
        }),
        throwsA(isA<TypeError>()),
      );
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

    test('SearchApplicationUsersQuery toJson uses API field names', () {
      final query = const SearchApplicationUsersQuery(
        page: 2,
        perPage: 10,
        sortOrder: 'asc',
        byNickname: 'alice',
        byEmail: 'a@example.com',
        byPhone: '123',
        byId: 'sqid1',
      );
      final json = query.toJson();
      expect(json['page'], 2);
      expect(json['per_page'], 10);
      expect(json['sort_order'], 'asc');
      expect(json['by_nickname'], 'alice');
      expect(json['by_email'], 'a@example.com');
      expect(json['by_phone'], '123');
      expect(json['by_id'], 'sqid1');
    });

    test('SearchApplicationUsersQuery fromJson', () {
      final query = SearchApplicationUsersQuery.fromJson({
        'page': 3,
        'per_page': 20,
        'by_nickname': 'bob',
      });
      expect(query.page, 3);
      expect(query.perPage, 20);
      expect(query.byNickname, 'bob');
      expect(query.byEmail, isNull);
    });

    test('ApplicationKey fromJson', () {
      final key = ApplicationKey.fromJson({
        'key_id': 'k1',
        'algorithm': 'RS256',
        'status': 'Active',
        'created_at': '2024-01-01T00:00:00Z',
        'activated_at': '2024-01-01T01:00:00Z',
        'retired_at': '2024-02-01T00:00:00Z',
        'expires_at': '2024-03-01T00:00:00Z',
        'revoked_at': null,
      });
      expect(key.keyId, 'k1');
      expect(key.algorithm, 'RS256');
      expect(key.status, 'Active');
      expect(key.createdAt, '2024-01-01T00:00:00Z');
      expect(key.activatedAt, '2024-01-01T01:00:00Z');
      expect(key.retiredAt, '2024-02-01T00:00:00Z');
      expect(key.expiresAt, '2024-03-01T00:00:00Z');
      expect(key.revokedAt, isNull);
    });

    test('ApplicationConfiguration matches the backend configuration schema',
        () {
      final configuration = ApplicationConfiguration.fromJson({
        'auth': {
          'token': {
            'issuer': 'OceanIAM',
            'audience': ['OceanIAM', 'example-api'],
          },
          'password': {
            'argon2': {'m_cost': 12288, 't_cost': 3, 'p_cost': 1},
          },
        },
        'registration': {'enabled': false},
        'development_accounts': {
          'enabled': true,
          'default_ttl_seconds': 3600,
          'max_ttl_seconds': 86400,
        },
        'oidc': {'allow_insecure_loopback_redirect_uris': false},
      });

      expect(configuration.auth.token.issuer, 'OceanIAM');
      expect(configuration.auth.token.audience, ['OceanIAM', 'example-api']);
      expect(configuration.auth.password.argon2.mCost, 12288);
      expect(configuration.auth.password.argon2.tCost, 3);
      expect(configuration.auth.password.argon2.pCost, 1);
      expect(configuration.registration.enabled, false);
      expect(configuration.developmentAccounts?.enabled, true);
      expect(configuration.developmentAccounts?.defaultTtlSeconds, 3600);
      expect(configuration.developmentAccounts?.maxTtlSeconds, 86400);
      expect(configuration.oidc?.allowInsecureLoopbackRedirectUris, false);
    });

    test('ApplicationConfiguration accepts legacy responses without oidc', () {
      final configuration = ApplicationConfiguration.fromJson({
        'auth': {
          'token': {'issuer': 'OceanIAM', 'audience': <String>[]},
          'password': {
            'argon2': {'m_cost': 12288, 't_cost': 3, 'p_cost': 1},
          },
        },
        'registration': {'enabled': false},
      });

      expect(configuration.oidc, isNull);
    });

    test('PatchApplicationConfiguration omits unchanged fields', () {
      const patch = PatchApplicationConfiguration(
        auth: PatchAuthConfiguration(
          token: PatchTokenConfiguration(issuer: 'Example'),
        ),
        developmentAccounts: PatchDevelopmentAccountsConfiguration(
          enabled: false,
        ),
        oidc: PatchOidcConfiguration(
          allowInsecureLoopbackRedirectUris: true,
        ),
      );

      expect(patch.toJson(), {
        'auth': {
          'token': {'issuer': 'Example'},
        },
        'development_accounts': {'enabled': false},
        'oidc': {'allow_insecure_loopback_redirect_uris': true},
      });
    });
  });

  group('OceanIAM Client', () {
    late OceanIAMClient client;
    late MockClient mockHttp;

    setUp(() {
      mockHttp = MockClient((request) async {
        if (request.url.path == '/auth/tokens' && request.method == 'POST') {
          return http.Response(jsonEncode({'jwt': 'test-jwt'}), 200);
        }
        if (request.url.path == '/tenants' && request.method == 'GET') {
          return http.Response(
            jsonEncode({
              'items': [
                {'id': 't1', 'comment': 'Tenant 1'},
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
        if (request.url.path == '/tenants/t1/applications/app1/users/search' &&
            request.method == 'GET') {
          expect(request.url.queryParameters['by_nickname'], 'alice');
          expect(request.url.queryParameters['page'], '1');
          expect(request.url.queryParameters['per_page'], '10');
          expect(request.url.queryParameters.containsKey('query'), isFalse);
          expect(
            request.url.queryParameters.containsKey('search_by'),
            isFalse,
          );
          return http.Response(
            jsonEncode({
              'items': [
                {
                  'id': 'u1',
                  'oidc_sub': '018f3f47-7b2f-7000-8000-000000000001',
                  'email': 'alice@example.com',
                  'phone': null,
                  'nickname': 'alice',
                },
              ],
              'page_info': {'has_next': false, 'total': 1},
            }),
            200,
          );
        }
        if (request.url.path == '/tenants/t1/keys' && request.method == 'GET') {
          return http.Response(
            jsonEncode({
              'items': [
                {
                  'key_id': 'k1',
                  'algorithm': 'RS256',
                  'status': 'Active',
                  'created_at': '2024-01-01T00:00:00Z',
                  'activated_at': '2024-01-01T01:00:00Z',
                  'retired_at': '2024-02-01T00:00:00Z',
                  'expires_at': '2024-03-01T00:00:00Z',
                  'revoked_at': null,
                },
              ],
              'page_info': {'has_next': false, 'total': 1},
            }),
            200,
          );
        }
        if (request.url.path == '/tenants/t1/keys' &&
            request.method == 'POST') {
          return http.Response('', 200);
        }
        if (request.url.path == '/tenants/t1/keys/k1' &&
            request.method == 'DELETE') {
          return http.Response('', 200);
        }
        if (request.url.path == '/tenants/t1/applications/app1/users/u1' &&
            request.method == 'DELETE') {
          return http.Response('', 200);
        }
        if (request.url.path == '/tenants/t1/applications/app1/configuration' &&
            request.method == 'GET') {
          return http.Response(
            jsonEncode({
              'configuration': {
                'auth': {
                  'token': {
                    'issuer': 'OceanIAM',
                    'audience': ['OceanIAM'],
                  },
                  'password': {
                    'argon2': {'m_cost': 12288, 't_cost': 3, 'p_cost': 1},
                  },
                },
                'registration': {'enabled': false},
                'development_accounts': {
                  'enabled': true,
                  'default_ttl_seconds': 3600,
                  'max_ttl_seconds': 86400,
                },
                'oidc': {
                  'allow_insecure_loopback_redirect_uris': false,
                },
              },
            }),
            200,
          );
        }
        if (request.url.path == '/tenants/t1/applications/app1/configuration' &&
            request.method == 'PATCH') {
          expect(jsonDecode(request.body), {
            'auth': {
              'token': {
                'issuer': 'Example',
                'audience': ['example-api'],
              },
            },
            'registration': {'enabled': true},
            'development_accounts': {
              'enabled': false,
              'max_ttl_seconds': 7200,
            },
            'oidc': {
              'allow_insecure_loopback_redirect_uris': true,
            },
          });
          return http.Response('', 200);
        }
        if (request.url.path.startsWith(
              '/tenants/t1/applications/app1/oidc-clients',
            ) &&
            !request.headers.containsKey('authorization')) {
          return http.Response(
            jsonEncode({'msg': 'missing authorization'}),
            203,
          );
        }
        if (request.url.path == '/tenants/t1/applications/app1/oidc-clients' &&
            request.method == 'GET') {
          expect(request.url.queryParameters['page'], '2');
          expect(request.url.queryParameters['per_page'], '10');
          return http.Response(
            jsonEncode({
              'items': [
                {
                  'client_id': 'client-sqid',
                  'application_id': 'app1',
                  'name': 'Web client',
                  'client_type': 'public',
                  'application_type': 'web',
                  'redirect_uris': ['https://client.example/callback'],
                  'created_at': '2026-09-07T00:00:00Z',
                },
              ],
              'page_info': {'has_next': false, 'total': 1},
            }),
            200,
          );
        }
        if (request.url.path == '/tenants/t1/applications/app1/oidc-clients' &&
            request.method == 'POST') {
          expect(jsonDecode(request.body), {
            'name': 'Web client',
            'redirect_uris': ['https://client.example/callback'],
          });
          return http.Response(
            jsonEncode({
              'client_id': 'client-sqid',
              'application_id': 'app1',
              'name': 'Web client',
              'client_type': 'public',
              'application_type': 'web',
              'redirect_uris': ['https://client.example/callback'],
              'created_at': '2026-09-07T00:00:00Z',
            }),
            200,
          );
        }
        if (request.url.path ==
                '/tenants/t1/applications/app1/oidc-clients/client-sqid' &&
            request.method == 'GET') {
          return http.Response(
            jsonEncode({
              'client_id': 'client-sqid',
              'application_id': 'app1',
              'name': 'Web client',
              'client_type': 'public',
              'application_type': 'web',
              'redirect_uris': ['https://client.example/callback'],
              'created_at': '2026-09-07T00:00:00Z',
            }),
            200,
          );
        }
        if (request.url.path ==
                '/tenants/t1/applications/app1/oidc-clients/client-sqid' &&
            request.method == 'PATCH') {
          expect(jsonDecode(request.body), {
            'redirect_uris': ['https://CLIENT.example/callback/%2Fraw'],
          });
          return http.Response(
            jsonEncode({
              'client_id': 'client-sqid',
              'application_id': 'app1',
              'name': 'Web client',
              'client_type': 'public',
              'application_type': 'web',
              'redirect_uris': ['https://CLIENT.example/callback/%2Fraw'],
              'created_at': '2026-09-07T00:00:00Z',
            }),
            200,
          );
        }
        if (request.url.path ==
                '/tenants/t1/applications/app1/oidc-clients/missing-client' &&
            request.method == 'DELETE') {
          return http.Response(
            jsonEncode({'msg': 'OIDC client not found'}),
            404,
          );
        }
        if (request.url.toString() ==
                'http://localhost:8000/tenants/t1/applications/app1/oidc-clients/client%3Frevision%3D1%2Fpart%23%252F' &&
            request.method == 'DELETE') {
          return http.Response('', 204);
        }
        if (request.url.path ==
                '/tenants/t1/applications/app1/oidc-clients/client-sqid' &&
            request.method == 'DELETE') {
          return http.Response('', 204);
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
      expect(resp.jwt, 'test-jwt');
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

    test('searchUsers returns paged response with by_* query params', () async {
      await client.signin('admin', 'password');
      final result = await client.searchUsers(
        't1',
        'app1',
        const SearchApplicationUsersQuery(
          page: 1,
          perPage: 10,
          byNickname: 'alice',
        ),
      );
      expect(result.items.length, 1);
      expect(result.items[0].nickname, 'alice');
      expect(
        result.items[0].oidcSub,
        '018f3f47-7b2f-7000-8000-000000000001',
      );
      expect(result.pageInfo.total, 1);
      expect(result.pageInfo.hasNext, false);
    });

    test('listKeys parses paged items', () async {
      await client.signin('admin', 'password');
      final keys = await client.listKeys('t1');
      expect(keys.length, 1);
      expect(keys[0].keyId, 'k1');
      expect(keys[0].status, 'Active');
      expect(keys[0].algorithm, 'RS256');
    });

    test('rotateKey and revokeKey succeed without body', () async {
      await client.signin('admin', 'password');
      await client.rotateKey('t1');
      await client.revokeKey('t1', 'k1');
    });

    test('deleteUser sends DELETE to the application-user endpoint', () async {
      await client.signin('admin', 'password');
      await client.deleteUser('t1', 'app1', 'u1');
    });

    test('gets and patches the full application configuration schema',
        () async {
      await client.signin('admin', 'password');
      final configuration = await client.getApplicationConfiguration(
        't1',
        'app1',
      );

      expect(configuration.auth.token.issuer, 'OceanIAM');
      expect(configuration.auth.password.argon2.mCost, 12288);
      expect(configuration.registration.enabled, false);
      expect(configuration.oidc?.allowInsecureLoopbackRedirectUris, false);

      await client.updateApplicationConfiguration(
        't1',
        'app1',
        const PatchApplicationConfiguration(
          auth: PatchAuthConfiguration(
            token: PatchTokenConfiguration(
              issuer: 'Example',
              audience: ['example-api'],
            ),
          ),
          registration: PatchRegistrationConfiguration(enabled: true),
          developmentAccounts: PatchDevelopmentAccountsConfiguration(
            enabled: false,
            maxTtlSeconds: 7200,
          ),
          oidc: PatchOidcConfiguration(
            allowInsecureLoopbackRedirectUris: true,
          ),
        ),
      );
    });

    test('OIDC methods surface missing authentication as OceanIAMError',
        () async {
      final missingAuth = isA<OceanIAMError>()
          .having((error) => error.statusCode, 'statusCode', 203)
          .having(
            (error) => error.message,
            'message',
            'missing authorization',
          );
      await expectLater(
        client.listOidcClients('t1', 'app1'),
        throwsA(missingAuth),
      );
      await expectLater(
        client.deleteOidcClient('t1', 'app1', 'client-sqid'),
        throwsA(missingAuth),
      );
    });

    test('manages OIDC clients with the application-scoped routes', () async {
      await client.signin('admin', 'password');

      final listed = await client.listOidcClients(
        't1',
        'app1',
        page: 2,
        pageSize: 10,
      );
      expect(listed.pageInfo.total, 1);
      expect(listed.items.single.clientId, 'client-sqid');

      final created = await client.createOidcClient(
        't1',
        'app1',
        name: 'Web client',
        redirectUris: ['https://client.example/callback'],
      );
      expect(created.clientType, 'public');
      expect(created.applicationType, 'web');

      final detail = await client.getOidcClient(
        't1',
        'app1',
        'client-sqid',
      );
      expect(detail.clientId, 'client-sqid');
      expect(detail.redirectUris, ['https://client.example/callback']);

      final patched = await client.patchOidcClient(
        't1',
        'app1',
        'client-sqid',
        const PatchOidcClientRequest(
          redirectUris: ['https://CLIENT.example/callback/%2Fraw'],
        ),
      );
      expect(patched.clientId, 'client-sqid');
      expect(patched.redirectUris, [
        'https://CLIENT.example/callback/%2Fraw',
      ]);

      await client.deleteOidcClient('t1', 'app1', 'client-sqid');
      await client.deleteOidcClient(
        't1',
        'app1',
        'client?revision=1/part#%2F',
      );

      await expectLater(
        client.deleteOidcClient('t1', 'app1', 'missing-client'),
        throwsA(
          isA<OceanIAMError>()
              .having((error) => error.statusCode, 'statusCode', 404)
              .having(
                (error) => error.message,
                'message',
                'OIDC client not found',
              ),
        ),
      );
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
          (_) async => http.Response(jsonEncode({'jwt': 'x'}), 200),
        ),
      );
      await localClient.signin('admin', 'pass');
      expect(localClient.isAuthenticated, true);
      localClient.dispose();
    });
  });
}
