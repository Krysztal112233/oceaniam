use oceaniam_auth::{Validation, decode, decode_header, jwks::JwkSet};
use oceaniam_database::{
    helper::oidc_authorization_codes::{IssueAuthorizationCodeInput, OidcAuthorizationCodesHelper},
    model::{
        oidc_authorization_codes,
        prelude::{OidcAuthorizationCodes, OidcAuthorizationTransactions, Users},
        sea_orm_active_enums::AuditType,
    },
};
use oceaniam_oidc::generate_authorization_code;
use reqwest::Response;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use url::form_urlencoded;
use uuid::Uuid;

use crate::support::{
    TestApp,
    oidc_drive::{
        CODE_VERIFIER, CodeRedirect, LoginFixture, USER_PASSWORD, assert_code_redirect,
        audits_of_type, create_user, drive_entry, drive_entry_without_nonce, login_form,
        post_login, seed_login_fixture, spawn_preview,
    },
    spawn_app_with_isolated_schema, sqid_to_uuid,
};

fn token_path(tenant_sqid: &str) -> String {
    format!("/oidc/{tenant_sqid}/token")
}

/// The RFC 6749 §4.1.3 token request parameter pairs every authorization-code exchange shares.
fn token_pairs<'a>(
    fixture: &'a LoginFixture,
    code: &'a str,
    verifier: &'a str,
) -> [(&'a str, &'a str); 5] {
    [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", fixture.redirect_uri.as_str()),
        ("client_id", fixture.client_id.as_str()),
        ("code_verifier", verifier),
    ]
}

fn token_form(fixture: &LoginFixture, code: &str, verifier: &str) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (name, value) in token_pairs(fixture, code, verifier) {
        serializer.append_pair(name, value);
    }
    serializer.finish()
}

fn token_form_with(
    fixture: &LoginFixture,
    code: &str,
    verifier: &str,
    name: &str,
    value: &str,
) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (parameter, parameter_value) in token_pairs(fixture, code, verifier) {
        let parameter_value = if parameter == name {
            value
        } else {
            parameter_value
        };
        serializer.append_pair(parameter, parameter_value);
    }
    serializer.finish()
}

async fn post_token(app: &TestApp, tenant_sqid: &str, form: String) -> Response {
    app.client
        .post(app.url(&token_path(tenant_sqid)))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form)
        .send()
        .await
        .expect("token request")
}

/// Posts a well-formed authorization_code exchange to the fixture's own tenant token path.
async fn exchange_code(
    app: &TestApp,
    fixture: &LoginFixture,
    code: &str,
    verifier: &str,
) -> Response {
    post_token(
        app,
        &fixture.tenant_sqid,
        token_form(fixture, code, verifier),
    )
    .await
}

/// Creates a password user, drives entry + login, and returns the user id, the transaction id,
/// and the issued plaintext code.
async fn drive_user_code(
    app: &TestApp,
    fixture: &LoginFixture,
    email: &str,
    state: &str,
) -> (Uuid, Uuid, CodeRedirect) {
    let user_id = create_user(app, fixture, email, USER_PASSWORD).await;
    let entry = drive_entry(app, fixture, state).await;
    let transaction_id = sqid_to_uuid(&entry.transaction_sqid);
    let response = post_login(
        app,
        fixture,
        &entry,
        login_form(email, USER_PASSWORD, &entry.csrf),
    )
    .await;
    let redirect = assert_code_redirect(response, fixture, &entry, state).await;
    (user_id, transaction_id, redirect)
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

async fn assert_token_error(response: Response, expected_error: &str) {
    assert_eq!(response.status(), 400);
    assert_common_headers(&response);
    let body: serde_json::Value = response.json().await.expect("token error body");
    assert_eq!(body["error"], expected_error);
    assert!(
        body.get("access_token").is_none() && body.get("id_token").is_none(),
        "an error must never carry tokens"
    );
}

async fn code_row(app: &TestApp, transaction_id: Uuid) -> oidc_authorization_codes::Model {
    let database = app.database().await;
    OidcAuthorizationCodes::find()
        .filter(oidc_authorization_codes::Column::TransactionId.eq(transaction_id))
        .one(&database)
        .await
        .expect("read authorization code row")
        .expect("authorization code row should exist")
}

async fn issue_token_audits(app: &TestApp) -> Vec<serde_json::Value> {
    audits_of_type(app, AuditType::OidcIssueToken).await
}

async fn tenant_jwks(app: &TestApp, tenant_sqid: &str) -> JwkSet {
    app.client
        .get(app.url(&format!("/oidc/{tenant_sqid}/jwks.json")))
        .send()
        .await
        .expect("tenant jwks request")
        .json()
        .await
        .expect("tenant jwks body")
}

// NOTE: AI-generated test
#[tokio::test]
async fn happy_path_issues_verifiable_jwt_access_and_id_tokens() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (user_id, transaction_id, redirect) =
        drive_user_code(&app, &fixture, "token-happy@example.com", "token-state").await;

    let response = exchange_code(&app, &fixture, &redirect.code, CODE_VERIFIER).await;
    assert_eq!(response.status(), 200);
    assert_common_headers(&response);
    let body: serde_json::Value = response.json().await.expect("token response body");
    assert_eq!(body["token_type"], "Bearer");
    assert_eq!(body["expires_in"], 3600);
    assert!(
        body.get("refresh_token").is_none(),
        "no refresh token may be issued this slice"
    );
    let access_token = body["access_token"].as_str().expect("access token");
    let id_token = body["id_token"].as_str().expect("id token");
    assert!(!access_token.contains(&redirect.code));
    assert!(!id_token.contains(&redirect.code));

    let database = app.database().await;
    let stored = OidcAuthorizationTransactions::find_by_id(transaction_id)
        .one(&database)
        .await
        .expect("read transaction")
        .expect("transaction should remain stored");
    let user = Users::find_by_id(user_id)
        .one(&database)
        .await
        .expect("read user")
        .expect("user should remain stored");

    // The code row is redeemed exactly once; the transaction row is untouched by the exchange.
    let code = code_row(&app, transaction_id).await;
    assert!(code.redeemed_at.is_some(), "the code must be burned");

    let jwks = tenant_jwks(&app, &fixture.tenant_sqid).await;

    // The access token verifies against the tenant JWKS with the registered audience and a
    // one-hour lifetime.
    let header = decode_header(access_token).expect("access token header");
    let key = jwks
        .decoding_key_for_kid(
            header
                .kid
                .as_deref()
                .expect("kid must reference the signing key"),
        )
        .expect("jwks must contain the signing kid");
    let mut validation = Validation::new(header.alg);
    validation.set_audience(&[fixture.client_id.as_str()]);
    let access = decode::<oceaniam_auth::jwt::Claim>(access_token, &key, &validation)
        .expect("access token must verify against the tenant JWKS")
        .claims;
    assert_eq!(access.sub, user.oidc_sub, "sub is the external oidc_sub");
    assert_eq!(access.exp - access.iat, 3600);
    assert_eq!(access.iss.as_deref(), Some(stored.issuer.as_str()));
    assert_eq!(access.aud, Some(vec![fixture.client_id.clone()]));
    assert_eq!(access.jti.get_version_num(), 7);

    // The ID token carries the OIDC claims: iss, sub, aud, exp, iat, the verbatim nonce, and
    // the transaction's auth_time.
    let id_header = decode_header(id_token).expect("id token header");
    let id_key = jwks
        .decoding_key_for_kid(id_header.kid.as_deref().expect("kid"))
        .expect("jwks must contain the id token kid");
    let mut id_validation = Validation::new(id_header.alg);
    id_validation.set_audience(&[fixture.client_id.as_str()]);
    let id_claims = decode::<serde_json::Value>(id_token, &id_key, &id_validation)
        .expect("id token must verify against the tenant JWKS")
        .claims;
    assert_eq!(id_claims["iss"], stored.issuer);
    assert_eq!(id_claims["sub"], user.oidc_sub.to_string());
    assert_eq!(id_claims["aud"], serde_json::json!([fixture.client_id]));
    assert_eq!(
        id_claims["exp"].as_i64().expect("exp") - id_claims["iat"].as_i64().expect("iat"),
        3600
    );
    assert_eq!(id_claims["nonce"], "private-nonce-sentinel");
    let authenticated_at = stored.authenticated_at.expect("authenticated transaction");
    assert_eq!(
        id_claims["auth_time"].as_i64().expect("auth_time"),
        authenticated_at.timestamp()
    );

    // The issuance is audited exactly once with identifiers only.
    let audits = issue_token_audits(&app).await;
    assert_eq!(audits.len(), 1, "token issuance must audit exactly once");
    let audit = &audits[0];
    assert_eq!(audit["kind"], "issue_token");
    assert_eq!(
        audit["data"]["application_id"],
        fixture.application_id.to_string()
    );
    assert_eq!(audit["data"]["transaction_id"], transaction_id.to_string());
    assert_eq!(audit["data"]["subject_id"], user_id.to_string());
    let audit_text = audit.to_string();
    assert!(!audit_text.contains(access_token));
    assert!(!audit_text.contains(id_token));
    assert!(!audit_text.contains(&redirect.code));
    assert!(!audit_text.contains(&fixture.redirect_uri));
}

// NOTE: AI-generated test
#[tokio::test]
async fn authorize_without_nonce_issues_id_token_without_nonce_key() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    create_user(&app, &fixture, "token-no-nonce@example.com", USER_PASSWORD).await;
    let state = "no-nonce-state";
    let entry = drive_entry_without_nonce(&app, &fixture, state).await;
    let response = post_login(
        &app,
        &fixture,
        &entry,
        login_form("token-no-nonce@example.com", USER_PASSWORD, &entry.csrf),
    )
    .await;
    let redirect = assert_code_redirect(response, &fixture, &entry, state).await;

    let response = exchange_code(&app, &fixture, &redirect.code, CODE_VERIFIER).await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.expect("token response body");
    let id_token = body["id_token"].as_str().expect("id token");

    // OIDC Core 1.0 §3.1.2.1: nonce is OPTIONAL; when the authorization request omitted it,
    // `IdTokenClaim::nonce` is None and the serde skip branch must omit the key entirely.
    let jwks = tenant_jwks(&app, &fixture.tenant_sqid).await;
    let id_header = decode_header(id_token).expect("id token header");
    let id_key = jwks
        .decoding_key_for_kid(id_header.kid.as_deref().expect("kid"))
        .expect("jwks must contain the id token kid");
    let mut id_validation = Validation::new(id_header.alg);
    id_validation.set_audience(&[fixture.client_id.as_str()]);
    let id_claims = decode::<serde_json::Value>(id_token, &id_key, &id_validation)
        .expect("id token must verify against the tenant JWKS")
        .claims;
    assert!(
        id_claims.get("nonce").is_none(),
        "the id token must omit the nonce key when the request carried none: {id_claims}"
    );
    assert_eq!(id_claims["aud"], serde_json::json!([fixture.client_id]));
    assert!(
        id_claims["auth_time"].is_i64(),
        "auth_time must still be present"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn replayed_code_is_invalid_grant() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, _, redirect) =
        drive_user_code(&app, &fixture, "token-replay@example.com", "replay-state").await;

    let first = exchange_code(&app, &fixture, &redirect.code, CODE_VERIFIER).await;
    assert_eq!(first.status(), 200);

    let replay = exchange_code(&app, &fixture, &redirect.code, CODE_VERIFIER).await;
    assert_token_error(replay, "invalid_grant").await;
    assert_eq!(issue_token_audits(&app).await.len(), 1);
}

// NOTE: AI-generated test
#[tokio::test]
async fn concurrent_redemption_has_exactly_one_winner() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, transaction_id, redirect) =
        drive_user_code(&app, &fixture, "token-race@example.com", "race-state").await;

    // Determinism: both exchanges race to consume the same code row.
    // `consume_authorization_code` is a single `UPDATE ... WHERE redeemed_at IS NULL`, which
    // takes a PostgreSQL row lock on the code row; the two transactions serialize on that lock,
    // and the loser re-evaluates the predicate against the winner's committed row, matches zero
    // rows, and fails as `invalid_grant`. Exactly one winner is therefore guaranteed; only
    // *which* request wins is nondeterministic, so every assertion below is order-insensitive.
    let (first, second) = tokio::join!(
        exchange_code(&app, &fixture, &redirect.code, CODE_VERIFIER),
        exchange_code(&app, &fixture, &redirect.code, CODE_VERIFIER),
    );
    let mut statuses = [first.status().as_u16(), second.status().as_u16()];
    statuses.sort_unstable();
    assert_eq!(
        statuses,
        [200, 400],
        "exactly one concurrent exchange may redeem the code"
    );
    let loser = if first.status() == 400 { first } else { second };
    assert_token_error(loser, "invalid_grant").await;

    assert_eq!(
        issue_token_audits(&app).await.len(),
        1,
        "only the winning exchange may audit"
    );
    let code = code_row(&app, transaction_id).await;
    assert!(
        code.redeemed_at.is_some(),
        "the code must be burned by the winner"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn code_issued_under_tenant_a_redeems_at_tenant_b_path_with_tenant_a_issuer() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, transaction_id, redirect) = drive_user_code(
        &app,
        &fixture,
        "token-cross-tenant@example.com",
        "cross-tenant-state",
    )
    .await;
    let other_tenant = app.api_create_tenant(&fixture.token).await;
    let other_tenant_sqid = other_tenant["id"].as_str().unwrap().to_string();

    // The path tenant scopes the URL namespace only: the consumed code's transaction row is the
    // authoritative tenant carrier (issuer snapshot and signing keybox) — the adjudicated design
    // documented in `endpoints/oidc/token.rs` ("The path tenant scopes the URL namespace only").
    // A code driven under tenant A must therefore still redeem — at tenant B's token path — with
    // tenant A's issuer and tenant A's signing keys.
    let response = post_token(
        &app,
        &other_tenant_sqid,
        token_form(&fixture, &redirect.code, CODE_VERIFIER),
    )
    .await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.expect("token response body");
    let id_token = body["id_token"].as_str().expect("id token");

    // The id token verifies against tenant A's JWKS (tenant B was never given a keybox usage
    // here) and carries tenant A's issuer snapshot.
    let jwks = tenant_jwks(&app, &fixture.tenant_sqid).await;
    let id_header = decode_header(id_token).expect("id token header");
    let id_key = jwks
        .decoding_key_for_kid(id_header.kid.as_deref().expect("kid"))
        .expect("tenant A jwks must contain the id token kid");
    let mut id_validation = Validation::new(id_header.alg);
    id_validation.set_audience(&[fixture.client_id.as_str()]);
    let id_claims = decode::<serde_json::Value>(id_token, &id_key, &id_validation)
        .expect("id token must verify against tenant A JWKS")
        .claims;
    let database = app.database().await;
    let stored = OidcAuthorizationTransactions::find_by_id(transaction_id)
        .one(&database)
        .await
        .expect("read transaction")
        .expect("transaction should remain stored");
    assert_eq!(id_claims["iss"], stored.issuer);
    assert!(
        stored
            .issuer
            .ends_with(&format!("/oidc/{}", fixture.tenant_sqid)),
        "iss must belong to tenant A, got {}",
        stored.issuer
    );
    assert!(
        !stored.issuer.contains(&other_tenant_sqid),
        "iss must never reference the path tenant B"
    );
    assert_eq!(issue_token_audits(&app).await.len(), 1);
}

// NOTE: AI-generated test
#[tokio::test]
async fn wrong_verifier_is_invalid_grant_and_burns_the_code() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, transaction_id, redirect) =
        drive_user_code(&app, &fixture, "token-pkce@example.com", "pkce-state").await;

    // RFC 7636 Appendix B verifier with the final character swapped: well-formed, mismatched.
    let wrong_verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXj";
    assert_ne!(wrong_verifier, CODE_VERIFIER);
    let response = exchange_code(&app, &fixture, &redirect.code, wrong_verifier).await;
    assert_token_error(response, "invalid_grant").await;

    // Fail closed: the failed exchange burned the code, so even the correct verifier cannot
    // redeem it afterwards, and no issuance is audited.
    let code = code_row(&app, transaction_id).await;
    assert!(code.redeemed_at.is_some(), "the burn must be committed");
    let retry = exchange_code(&app, &fixture, &redirect.code, CODE_VERIFIER).await;
    assert_token_error(retry, "invalid_grant").await;
    assert!(issue_token_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn mismatched_client_id_is_invalid_grant_and_burns_the_code() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, transaction_id, redirect) =
        drive_user_code(&app, &fixture, "token-client@example.com", "client-state").await;

    let form = token_form_with(
        &fixture,
        &redirect.code,
        CODE_VERIFIER,
        "client_id",
        "some-other-client",
    );
    let response = post_token(&app, &fixture.tenant_sqid, form).await;
    assert_token_error(response, "invalid_grant").await;

    let code = code_row(&app, transaction_id).await;
    assert!(code.redeemed_at.is_some(), "the burn must be committed");
    assert!(issue_token_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn mismatched_redirect_uri_is_invalid_grant_and_burns_the_code() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, transaction_id, redirect) = drive_user_code(
        &app,
        &fixture,
        "token-redirect@example.com",
        "redirect-state",
    )
    .await;

    let form = token_form_with(
        &fixture,
        &redirect.code,
        CODE_VERIFIER,
        "redirect_uri",
        "https://client.example/other-callback",
    );
    let response = post_token(&app, &fixture.tenant_sqid, form).await;
    assert_token_error(response, "invalid_grant").await;

    let code = code_row(&app, transaction_id).await;
    assert!(code.redeemed_at.is_some(), "the burn must be committed");
    assert!(issue_token_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn unsupported_grant_type_never_touches_the_code() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, transaction_id, redirect) =
        drive_user_code(&app, &fixture, "token-grant@example.com", "grant-state").await;

    let form = token_form_with(
        &fixture,
        &redirect.code,
        CODE_VERIFIER,
        "grant_type",
        "refresh_token",
    );
    let response = post_token(&app, &fixture.tenant_sqid, form).await;
    assert_token_error(response, "unsupported_grant_type").await;

    // Request-shape validation precedes redemption: the code stays redeemable.
    let code = code_row(&app, transaction_id).await;
    assert!(code.redeemed_at.is_none(), "the code must not be burned");
    let exchange = exchange_code(&app, &fixture, &redirect.code, CODE_VERIFIER).await;
    assert_eq!(exchange.status(), 200);
}

// NOTE: AI-generated test
#[tokio::test]
async fn expired_code_is_invalid_grant() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (user_id, transaction_id, _) =
        drive_user_code(&app, &fixture, "token-expired@example.com", "expired-state").await;

    // Insert an already-expired code (negative TTL) bound to the completed transaction.
    let (plaintext, digest) = generate_authorization_code();
    let database = app.database().await;
    OidcAuthorizationCodes::issue_authorization_code(
        IssueAuthorizationCodeInput {
            transaction_id,
            code_digest: digest,
            redirect_uri: fixture.redirect_uri.clone(),
            subject_id: user_id,
        },
        chrono::Duration::seconds(-60),
        &database,
    )
    .await
    .expect("insert expired authorization code");

    let response = exchange_code(&app, &fixture, &plaintext, CODE_VERIFIER).await;
    assert_token_error(response, "invalid_grant").await;
    assert!(issue_token_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn unknown_code_is_invalid_grant() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;

    let (plaintext, _) = generate_authorization_code();
    let response = exchange_code(&app, &fixture, &plaintext, CODE_VERIFIER).await;
    assert_token_error(response, "invalid_grant").await;
    assert!(issue_token_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn missing_parameters_and_malformed_tenant_are_invalid_request() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;

    let response = post_token(
        &app,
        &fixture.tenant_sqid,
        "grant_type=authorization_code".to_owned(),
    )
    .await;
    assert_token_error(response, "invalid_request").await;

    let (plaintext, _) = generate_authorization_code();
    let response = post_token(
        &app,
        "@@not-a-sqid@@",
        token_form(&fixture, &plaintext, CODE_VERIFIER),
    )
    .await;
    assert_token_error(response, "invalid_request").await;
}

// NOTE: AI-generated test
#[tokio::test]
async fn transport_rejections_are_plain_text() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let url = app.url(&token_path(&fixture.tenant_sqid));

    let oversized = app
        .client
        .post(&url)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(vec![b'x'; 8 * 1024 + 1])
        .send()
        .await
        .expect("oversized token body");
    assert_eq!(oversized.status(), 413);
    assert_eq!(
        oversized
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/plain; charset=utf-8")
    );
    assert_eq!(
        oversized.text().await.expect("read 413 body"),
        "token request body is too large"
    );

    let wrong_media = app
        .client
        .post(&url)
        .header("content-type", "application/json")
        .body("{}")
        .send()
        .await
        .expect("wrong media type token request");
    assert_eq!(wrong_media.status(), 415);
    assert_eq!(
        wrong_media
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/plain; charset=utf-8")
    );
    assert_eq!(
        wrong_media.text().await.expect("read 415 body"),
        "token request must be form-encoded"
    );

    // Transport rejections precede redemption: no issuance may be audited.
    assert!(issue_token_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn disabled_preview_gate_is_404() {
    let app = spawn_app_with_isolated_schema().await;
    let tenant_sqid = oceaniam_common::sqid::Sqid::from(Uuid::now_v7()).to_string();
    let response = post_token(
        &app,
        &tenant_sqid,
        "grant_type=authorization_code&code=whatever".to_owned(),
    )
    .await;
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
#[test]
fn openapi_documents_the_token_method() {
    let specification = oceaniam::app::build_openapi_spec();
    let path = specification
        .paths
        .paths
        .get("/oidc/{tenant_sqid}/token")
        .expect("token path should be documented");
    assert!(path.post.is_some());
    assert!(path.get.is_none());
}
