use oceaniam_auth::{decode_header, jwt::Claim};
use oceaniam_database::{
    helper::oidc_clients::OidcClientsHelper,
    model::prelude::{OidcClients, Users},
};
use reqwest::Response;
use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::support::{
    TestApp,
    oidc_drive::{LoginFixture, drive_user_token, seed_login_fixture, spawn_preview},
    spawn_app_with_isolated_schema, sqid_to_uuid, uuid_to_sqid,
};

fn userinfo_path(tenant_sqid: &str) -> String {
    format!("/oidc/{tenant_sqid}/userinfo")
}

/// The tenant issuer under the hard-coded test public base URL
/// (`OCEANIAM_PUBLIC_BASE_URL=http://localhost:8000` in `tests/support/mod.rs`).
fn tenant_issuer(tenant_sqid: &str) -> String {
    format!("http://localhost:8000/oidc/{tenant_sqid}")
}

fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

async fn get_userinfo(app: &TestApp, tenant_sqid: &str, authorization: Option<&str>) -> Response {
    let request = app.client.get(app.url(&userinfo_path(tenant_sqid)));
    let request = match authorization {
        Some(value) => request.header("authorization", value),
        None => request,
    };
    request.send().await.expect("userinfo GET request")
}

async fn post_userinfo(app: &TestApp, tenant_sqid: &str, authorization: Option<&str>) -> Response {
    let request = app.client.post(app.url(&userinfo_path(tenant_sqid)));
    let request = match authorization {
        Some(value) => request.header("authorization", value),
        None => request,
    };
    request.send().await.expect("userinfo POST request")
}

fn assert_common_headers(response: &Response) {
    for (name, expected) in [
        ("cache-control", "no-store"),
        ("pragma", "no-cache"),
        ("referrer-policy", "no-referrer"),
        ("x-content-type-options", "nosniff"),
    ] {
        assert_eq!(
            response
                .headers()
                .get(name)
                .and_then(|value| value.to_str().ok()),
            Some(expected),
            "header {name}"
        );
    }
}

/// RFC 6750 §3.1: every presented-token failure collapses into one byte-identical 401, and
/// the response must never echo the presented token.
async fn assert_invalid_token(response: Response, tenant_sqid: &str, presented: Option<&str>) {
    assert_eq!(response.status(), 401);
    assert_common_headers(&response);
    let challenge = format!(
        "Bearer realm=\"{}\", error=\"invalid_token\"",
        tenant_issuer(tenant_sqid)
    );
    assert_eq!(
        response
            .headers()
            .get("www-authenticate")
            .and_then(|value| value.to_str().ok()),
        Some(challenge.as_str()),
        "WWW-Authenticate challenge"
    );
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/plain; charset=utf-8")
    );
    let body = response.text().await.expect("read 401 body");
    assert_eq!(body, "invalid access token");
    if let Some(token) = presented {
        assert!(
            !body.contains(token),
            "the error response must never echo the presented token"
        );
    }
}

/// RFC 6750 §3.1: no token presented, so the challenge carries the realm but no error code.
async fn assert_missing_token_challenge(response: Response, tenant_sqid: &str) {
    assert_eq!(response.status(), 401);
    assert_common_headers(&response);
    let challenge = format!("Bearer realm=\"{}\"", tenant_issuer(tenant_sqid));
    assert_eq!(
        response
            .headers()
            .get("www-authenticate")
            .and_then(|value| value.to_str().ok()),
        Some(challenge.as_str()),
        "WWW-Authenticate challenge must carry no error code"
    );
    let body = response.text().await.expect("read 401 body");
    assert_eq!(body, "missing bearer token");
}

async fn user_oidc_sub(app: &TestApp, user_id: Uuid) -> Uuid {
    let database = app.database().await;
    Users::find_by_id(user_id)
        .one(&database)
        .await
        .expect("read user")
        .expect("user should exist")
        .oidc_sub
}

fn claim_for(
    fixture: &LoginFixture,
    oidc_sub: Uuid,
    iat: i64,
    exp: i64,
    audiences: Vec<String>,
) -> Claim {
    Claim {
        sub: oidc_sub,
        exp,
        iat,
        iss: Some(tenant_issuer(&fixture.tenant_sqid)),
        aud: Some(audiences),
        jti: Uuid::now_v7(),
    }
}

async fn sign_claim<C>(app: &TestApp, fixture: &LoginFixture, claim: C) -> String
where
    C: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + 'static,
{
    app.state
        .keyboxes
        .clone()
        .sign_assembled_jwt(sqid_to_uuid(&fixture.tenant_sqid), claim)
        .await
        .expect("sign custom claim")
        .jwt
}

/// Claims shape without any `aud` field: `Claim` serializes `aud: None` as `"aud": null`,
/// so a true absence needs its own type.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct AudlessClaim {
    sub: Uuid,
    exp: i64,
    iat: i64,
    iss: String,
    jti: Uuid,
}

/// Claims shape without any `iss` field (same true-absence reasoning as [`AudlessClaim`]).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct IsslessClaim {
    sub: Uuid,
    exp: i64,
    iat: i64,
    aud: Vec<String>,
    jti: Uuid,
}

// NOTE: AI-generated test
#[tokio::test]
async fn happy_path_get_returns_sub_only() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (user_id, access_token) = drive_user_token(
        &app,
        &fixture,
        "userinfo-happy@example.com",
        "userinfo-state",
    )
    .await;

    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(&access_token))).await;
    assert_eq!(response.status(), 200);
    assert_common_headers(&response);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/json")
    );
    let body: serde_json::Value = response.json().await.expect("userinfo body");
    let object = body.as_object().expect("userinfo body should be an object");
    assert_eq!(object.len(), 1, "only `sub` may be returned: {body}");
    assert_eq!(
        body["sub"],
        user_oidc_sub(&app, user_id).await.to_string(),
        "sub is the user's stable external oidc_sub"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn happy_path_post_returns_sub_and_ignores_the_body() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (user_id, access_token) = drive_user_token(
        &app,
        &fixture,
        "userinfo-post@example.com",
        "userinfo-post-state",
    )
    .await;

    let expected_sub = user_oidc_sub(&app, user_id).await.to_string();

    // Plain POST is equivalent to GET (OIDC Core 5.3.1 MUST support both).
    let response = post_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(&access_token))).await;
    assert_eq!(response.status(), 200);
    assert_common_headers(&response);
    let body: serde_json::Value = response.json().await.expect("userinfo body");
    assert_eq!(body["sub"], expected_sub);

    // RFC 6750 §2.1 header transport only: a form body — even one carrying a token parameter —
    // is ignored entirely.
    let response = app
        .client
        .post(app.url(&userinfo_path(&fixture.tenant_sqid)))
        .header("authorization", bearer(&access_token))
        .header("content-type", "application/x-www-form-urlencoded")
        .body("access_token=body-tokens-are-never-consulted")
        .send()
        .await
        .expect("userinfo POST request");
    assert_eq!(response.status(), 200);
    assert_common_headers(&response);
    let body: serde_json::Value = response.json().await.expect("userinfo body");
    assert_eq!(body["sub"], expected_sub);
}

// NOTE: AI-generated test
#[tokio::test]
async fn missing_authorization_header_is_challenged_without_error_code() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, access_token) = drive_user_token(
        &app,
        &fixture,
        "userinfo-missing@example.com",
        "missing-state",
    )
    .await;

    let response = get_userinfo(&app, &fixture.tenant_sqid, None).await;
    assert_missing_token_challenge(response, &fixture.tenant_sqid).await;

    // A token smuggled through the query string is never consulted (RFC 6750 §2.1 header
    // transport only), so the request is still answered as if no token were presented.
    let response = app
        .client
        .get(app.url(&format!(
            "{}?access_token={access_token}",
            userinfo_path(&fixture.tenant_sqid)
        )))
        .send()
        .await
        .expect("userinfo GET with query token");
    assert_missing_token_challenge(response, &fixture.tenant_sqid).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn non_bearer_garbage_and_kidless_tokens_are_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;

    let response = get_userinfo(&app, &fixture.tenant_sqid, Some("Basic dXNlcjpwYXNz")).await;
    assert_invalid_token(response, &fixture.tenant_sqid, None).await;

    let garbage = "not-a-jwt";
    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(garbage))).await;
    assert_invalid_token(response, &fixture.tenant_sqid, Some(garbage)).await;

    // Well-formed JWT header and payload, but the header carries no `kid`.
    let kidless = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.e30.signature";
    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(kidless))).await;
    assert_invalid_token(response, &fixture.tenant_sqid, Some(kidless)).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn tampered_payload_segment_is_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, access_token) = drive_user_token(
        &app,
        &fixture,
        "userinfo-tampered@example.com",
        "tampered-state",
    )
    .await;

    // Flip one base64url character in the middle of the payload segment: the signature can no
    // longer verify (or the payload no longer decodes), so the token is rejected.
    let segments: Vec<&str> = access_token.split('.').collect();
    assert_eq!(segments.len(), 3, "a JWS compact token has three segments");
    let mut payload: Vec<char> = segments[1].chars().collect();
    let middle = payload.len() / 2;
    payload[middle] = if payload[middle] == 'A' { 'B' } else { 'A' };
    let tampered_payload: String = payload.into_iter().collect();
    let tampered = format!("{}.{}.{}", segments[0], tampered_payload, segments[2]);
    assert_ne!(tampered, access_token);

    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(&tampered))).await;
    assert_invalid_token(response, &fixture.tenant_sqid, Some(&tampered)).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn expired_token_is_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (user_id, _) = drive_user_token(
        &app,
        &fixture,
        "userinfo-expired@example.com",
        "expired-state",
    )
    .await;

    // Well past jsonwebtoken's default 60-second leeway.
    let now = chrono::Utc::now().timestamp();
    let token = sign_claim(
        &app,
        &fixture,
        claim_for(
            &fixture,
            user_oidc_sub(&app, user_id).await,
            now - 7200,
            now - 3600,
            vec![fixture.client_id.clone()],
        ),
    )
    .await;

    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(&token))).await;
    assert_invalid_token(response, &fixture.tenant_sqid, Some(&token)).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn unregistered_audience_is_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (user_id, _) = drive_user_token(
        &app,
        &fixture,
        "userinfo-audience@example.com",
        "audience-state",
    )
    .await;

    // Signature, issuer, and expiry are all valid; only the audience fails to resolve to a
    // registered client of the path tenant.
    let now = chrono::Utc::now().timestamp();
    let token = sign_claim(
        &app,
        &fixture,
        claim_for(
            &fixture,
            user_oidc_sub(&app, user_id).await,
            now,
            now + 3600,
            vec!["unknown-client".to_owned()],
        ),
    )
    .await;

    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(&token))).await;
    assert_invalid_token(response, &fixture.tenant_sqid, Some(&token)).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn cross_tenant_token_is_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, access_token) = drive_user_token(
        &app,
        &fixture,
        "userinfo-cross-tenant@example.com",
        "cross-tenant-state",
    )
    .await;
    let other_tenant = app.api_create_tenant(&fixture.token).await;
    let other_tenant_sqid = other_tenant["id"].as_str().unwrap().to_string();

    // Tenant A's token presented at tenant B's UserInfo path: tenant B exists, but its JWKS
    // knows neither the signing key nor the issuer, so the token is rejected.
    let response = get_userinfo(&app, &other_tenant_sqid, Some(&bearer(&access_token))).await;
    assert_invalid_token(response, &other_tenant_sqid, Some(&access_token)).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn revoked_signing_key_is_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, access_token) = drive_user_token(
        &app,
        &fixture,
        "userinfo-revoked@example.com",
        "revoked-state",
    )
    .await;

    // The tenant JWKS excludes revoked keys, so revoking the signing key invalidates every
    // token it signed.
    let header = decode_header(&access_token).expect("access token header");
    let kid = header.kid.expect("access token must carry a kid");
    app.state
        .keyboxes
        .revoke_key(
            sqid_to_uuid(&fixture.tenant_sqid),
            Uuid::parse_str(&kid).expect("kid is a uuid"),
        )
        .await
        .expect("revoke the signing key");

    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(&access_token))).await;
    assert_invalid_token(response, &fixture.tenant_sqid, Some(&access_token)).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn deleted_subject_is_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (user_id, access_token) = drive_user_token(
        &app,
        &fixture,
        "userinfo-deleted@example.com",
        "deleted-state",
    )
    .await;

    // The access token outlives the user row; subject resolution fails closed.
    let response = app
        .api_delete_application_user(
            &fixture.token,
            &fixture.tenant_sqid,
            &fixture.application_sqid,
            &uuid_to_sqid(user_id),
        )
        .await;
    assert!(
        response.status().is_success(),
        "delete application user should succeed (got {})",
        response.status()
    );

    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(&access_token))).await;
    assert_invalid_token(response, &fixture.tenant_sqid, Some(&access_token)).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn disabled_preview_gate_is_404() {
    let app = spawn_app_with_isolated_schema().await;
    let tenant_sqid = oceaniam_common::sqid::Sqid::from(Uuid::now_v7()).to_string();

    let response = get_userinfo(&app, &tenant_sqid, None).await;
    assert_eq!(response.status(), 404);
    // The PreviewEnabled extractor answers with a plain-text "not found" body, which
    // discriminates the gate-closed 404 from a route-absent 404.
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/plain; charset=utf-8")
    );
    let body = response.text().await.expect("read gate response body");
    assert_eq!(body, "not found");
}

// NOTE: AI-generated test
#[tokio::test]
async fn unsupported_methods_are_405_with_allow() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;

    // OIDC Core §5.3.1 mandates GET and POST only; axum's method router answers everything
    // else with a bare 405 plus the Allow header.
    for method in [reqwest::Method::PUT, reqwest::Method::DELETE] {
        let response = app
            .client
            .request(
                method.clone(),
                app.url(&userinfo_path(&fixture.tenant_sqid)),
            )
            .send()
            .await
            .expect("userinfo request with unsupported method");
        assert_eq!(response.status(), 405, "method {method}");
        let allow = response
            .headers()
            .get("allow")
            .and_then(|value| value.to_str().ok())
            .expect("405 must carry an Allow header");
        assert!(allow.contains("GET"), "Allow must list GET: {allow}");
        assert!(allow.contains("POST"), "Allow must list POST: {allow}");
    }
}

// NOTE: AI-generated test
#[tokio::test]
async fn malformed_tenant_is_400_and_unknown_tenant_is_404() {
    let app = spawn_preview().await;

    // A malformed tenant identifier has no derivable issuer realm, so the RFC 6750 §3.1
    // challenge carries `invalid_request` without a realm.
    let response = get_userinfo(&app, "@@not-a-sqid@@", Some("Bearer whatever")).await;
    assert_eq!(response.status(), 400);
    assert_common_headers(&response);
    assert_eq!(
        response
            .headers()
            .get("www-authenticate")
            .and_then(|value| value.to_str().ok()),
        Some("Bearer error=\"invalid_request\"")
    );
    let body = response.text().await.expect("read 400 body");
    assert_eq!(body, "invalid tenant identifier");

    // A well-formed but absent tenant resolves through the system-protected lookup before any
    // token work: 404, namespace parity with the JWKS route, and no challenge header.
    let unknown = oceaniam_common::sqid::Sqid::from(Uuid::now_v7()).to_string();
    let response = get_userinfo(&app, &unknown, Some("Bearer whatever")).await;
    assert_eq!(response.status(), 404);
    assert!(
        response.headers().get("www-authenticate").is_none(),
        "a tenant-namespace 404 carries no bearer challenge"
    );
    let body = response.text().await.expect("read 404 body");
    assert_eq!(body, "not found");
}

// NOTE: AI-generated test
#[tokio::test]
async fn non_utf8_authorization_header_is_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;

    // A present but undecodable Authorization header still presented credentials, so RFC 6750
    // §3.1 requires the `invalid_token` challenge rather than the missing-token one.
    let response = app
        .client
        .get(app.url(&userinfo_path(&fixture.tenant_sqid)))
        .header(
            "authorization",
            reqwest::header::HeaderValue::from_bytes(b"Bearer \xff\xfe")
                .expect("opaque header value"),
        )
        .send()
        .await
        .expect("userinfo GET request");
    assert_invalid_token(response, &fixture.tenant_sqid, None).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn missing_audience_claim_is_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (user_id, _) = drive_user_token(
        &app,
        &fixture,
        "userinfo-no-aud@example.com",
        "no-aud-state",
    )
    .await;

    // Signature, issuer, and expiry are all valid; the token carries no `aud` field at all.
    let now = chrono::Utc::now().timestamp();
    let token = sign_claim(
        &app,
        &fixture,
        AudlessClaim {
            sub: user_oidc_sub(&app, user_id).await,
            exp: now + 3600,
            iat: now,
            iss: tenant_issuer(&fixture.tenant_sqid),
            jti: Uuid::now_v7(),
        },
    )
    .await;

    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(&token))).await;
    assert_invalid_token(response, &fixture.tenant_sqid, Some(&token)).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn empty_audience_claim_is_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (user_id, _) = drive_user_token(
        &app,
        &fixture,
        "userinfo-empty-aud@example.com",
        "empty-aud-state",
    )
    .await;

    let now = chrono::Utc::now().timestamp();
    let token = sign_claim(
        &app,
        &fixture,
        claim_for(
            &fixture,
            user_oidc_sub(&app, user_id).await,
            now,
            now + 3600,
            vec![],
        ),
    )
    .await;

    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(&token))).await;
    assert_invalid_token(response, &fixture.tenant_sqid, Some(&token)).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn cross_tenant_audience_is_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (user_id, _) = drive_user_token(
        &app,
        &fixture,
        "userinfo-cross-aud@example.com",
        "cross-aud-state",
    )
    .await;

    // Register a client under a DIFFERENT tenant.
    let other_tenant = app.api_create_tenant(&fixture.token).await;
    let other_tenant_sqid = other_tenant["id"].as_str().unwrap().to_string();
    let other_application = app
        .api_create_application(&fixture.token, &other_tenant_sqid)
        .await;
    let other_application_id = sqid_to_uuid(other_application["application_id"].as_str().unwrap());
    let other_client_id = format!("other-tenant-client-{}", Uuid::now_v7().simple());
    let database = app.database().await;
    OidcClients::create_client(
        Uuid::now_v7(),
        other_application_id,
        other_client_id.clone(),
        "Other tenant client".to_owned(),
        &database,
    )
    .await
    .expect("create other tenant client");

    // Tenant A signs the token, but the audience names tenant B's client: the tenant-scoped
    // client resolution must miss.
    let now = chrono::Utc::now().timestamp();
    let token = sign_claim(
        &app,
        &fixture,
        claim_for(
            &fixture,
            user_oidc_sub(&app, user_id).await,
            now,
            now + 3600,
            vec![other_client_id],
        ),
    )
    .await;

    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(&token))).await;
    assert_invalid_token(response, &fixture.tenant_sqid, Some(&token)).await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn missing_issuer_claim_is_invalid_token() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (user_id, _) = drive_user_token(
        &app,
        &fixture,
        "userinfo-no-iss@example.com",
        "no-iss-state",
    )
    .await;

    // jsonwebtoken only validates `iss` when the token carries one, so the endpoint requires
    // the claim outright; an issuer-less token is rejected.
    let now = chrono::Utc::now().timestamp();
    let token = sign_claim(
        &app,
        &fixture,
        IsslessClaim {
            sub: user_oidc_sub(&app, user_id).await,
            exp: now + 3600,
            iat: now,
            aud: vec![fixture.client_id.clone()],
            jti: Uuid::now_v7(),
        },
    )
    .await;

    let response = get_userinfo(&app, &fixture.tenant_sqid, Some(&bearer(&token))).await;
    assert_invalid_token(response, &fixture.tenant_sqid, Some(&token)).await;
}

// NOTE: AI-generated test
#[test]
fn openapi_spec_contains_oidc_userinfo_paths() {
    let specification = oceaniam::app::build_openapi_spec();
    let path = specification
        .paths
        .paths
        .get("/oidc/{tenant_sqid}/userinfo")
        .expect("userinfo path should be documented");
    assert!(path.get.is_some(), "GET userinfo must be documented");
    assert!(path.post.is_some(), "POST userinfo must be documented");
}
