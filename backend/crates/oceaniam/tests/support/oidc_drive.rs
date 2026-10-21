//! Shared OIDC authorization-flow drive fixtures, moved verbatim from
//! `tests/api/oidc/login.rs` so the token-endpoint tests can drive the same
//! entry → login → code chain. `login.rs` re-imports these; `drive_user_token` extends the
//! chain through the token exchange for the UserInfo endpoint tests.
#![allow(dead_code)]

use argon2::Argon2;
use axum_extra::extract::cookie::Cookie;
use oceaniam_common::config::BackendConfig;
use oceaniam_credential::{CredentialVault, Totp};
use oceaniam_database::{
    helper::{
        oidc_clients::OidcClientsHelper,
        users::{CreateUserOpts, UserHelper},
    },
    model::{oidc_clients, prelude::Audits, prelude::Users, sea_orm_active_enums::AuditType},
};
use reqwest::Response;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use url::form_urlencoded;
use uuid::Uuid;

use crate::support::{TestApp, spawn_app_with_isolated_schema_configured, sqid_to_uuid};

pub const CODE_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
/// RFC 7636 Appendix B verifier; pairs with [`CODE_CHALLENGE`].
pub const CODE_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
pub const USER_PASSWORD: &str = "CorrectHorse42!";
// Deterministic non-zero test KEK, identical to the fixture key in `tests/support/mod.rs`.
pub const TEST_MASTER_KEY: [u8; 32] = [
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
];

pub struct LoginFixture {
    pub tenant_sqid: String,
    pub application_id: Uuid,
    pub application_sqid: String,
    pub oidc_client_id: Uuid,
    pub client_id: String,
    pub redirect_uri: String,
    pub token: String,
}

pub struct EntryContext {
    pub cookie_name: String,
    pub cookie_value: String,
    pub csrf: String,
    pub transaction_sqid: String,
}

pub async fn spawn_preview() -> TestApp {
    spawn_app_with_isolated_schema_configured(|config: &mut BackendConfig| {
        config.oidc.authorization_entry_preview_enabled = true;
    })
    .await
}

/// Lists the payloads of every audit row of the given type.
pub async fn audits_of_type(app: &TestApp, audit_type: AuditType) -> Vec<serde_json::Value> {
    let database = app.database().await;
    Audits::find()
        .filter(oceaniam_database::model::audits::Column::AuditType.eq(audit_type))
        .all(&database)
        .await
        .expect("list audits by type")
        .into_iter()
        .map(|audit| audit.payload)
        .collect()
}

pub async fn seed_login_fixture(app: &TestApp) -> LoginFixture {
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_sqid = tenant["id"].as_str().unwrap().to_string();
    let application = app.api_create_application(&token, &tenant_sqid).await;
    let application_sqid = application["application_id"].as_str().unwrap().to_string();
    let application_id = sqid_to_uuid(&application_sqid);

    let database = app.database().await;
    let oidc_client_id = Uuid::now_v7();
    let client_id = format!("login-client-{}", oidc_client_id.simple());
    let redirect_uri = "https://client.example/callback".to_owned();
    oidc_clients::Entity::create_client(
        oidc_client_id,
        application_id,
        client_id.clone(),
        "Login test client".to_owned(),
        &database,
    )
    .await
    .expect("create login test client");
    oidc_clients::Entity::create_redirect_uris(
        oidc_client_id,
        vec![redirect_uri.clone()],
        &database,
    )
    .await
    .expect("create login test redirect");

    LoginFixture {
        tenant_sqid,
        application_id,
        application_sqid,
        oidc_client_id,
        client_id,
        redirect_uri,
        token,
    }
}

fn authorize_query_with_nonce(fixture: &LoginFixture, state: &str, nonce: Option<&str>) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (name, value) in [
        ("response_type", "code"),
        ("client_id", fixture.client_id.as_str()),
        ("redirect_uri", fixture.redirect_uri.as_str()),
        ("scope", "openid"),
        ("state", state),
        ("code_challenge", CODE_CHALLENGE),
        ("code_challenge_method", "S256"),
    ] {
        serializer.append_pair(name, value);
    }
    if let Some(nonce) = nonce {
        serializer.append_pair("nonce", nonce);
    }
    serializer.finish()
}

fn html_attribute(body: &str, name: &str) -> String {
    let marker = format!("{name}=\"");
    let start = body.find(&marker).expect("HTML attribute should exist") + marker.len();
    let end = body[start..]
        .find('"')
        .map(|offset| start + offset)
        .expect("HTML attribute should close");
    body[start..end].to_owned()
}

pub async fn drive_entry(app: &TestApp, fixture: &LoginFixture, state: &str) -> EntryContext {
    drive_entry_with_nonce(app, fixture, state, Some("private-nonce-sentinel")).await
}

/// Drives the authorization entry without the optional `nonce` parameter (OIDC Core 1.0
/// §3.1.2.1: `nonce` is OPTIONAL for the authorization code flow).
pub async fn drive_entry_without_nonce(
    app: &TestApp,
    fixture: &LoginFixture,
    state: &str,
) -> EntryContext {
    drive_entry_with_nonce(app, fixture, state, None).await
}

async fn drive_entry_with_nonce(
    app: &TestApp,
    fixture: &LoginFixture,
    state: &str,
    nonce: Option<&str>,
) -> EntryContext {
    let response = app
        .client
        .get(app.url(&format!(
            "/oidc/{}/authorize?{}",
            fixture.tenant_sqid,
            authorize_query_with_nonce(fixture, state, nonce)
        )))
        .send()
        .await
        .expect("entry authorization request");
    assert_eq!(response.status(), 200, "entry should render the login form");
    let cookie_header = response
        .headers()
        .get("set-cookie")
        .expect("entry should set the binding cookie")
        .to_str()
        .expect("binding cookie should be ASCII")
        .to_owned();
    let body = response.text().await.expect("read entry HTML");

    let cookie = Cookie::parse(cookie_header)
        .expect("binding cookie should parse")
        .into_owned();
    let transaction_sqid = html_attribute(&body, "data-transaction-sqid");
    assert_eq!(
        cookie.name(),
        format!("oceaniam_oidc_binding_{transaction_sqid}")
    );
    assert!(
        body.contains(&format!(
            "<form method=\"post\" action=\"/oidc/{}/authorize/{transaction_sqid}/login\"",
            fixture.tenant_sqid,
        )),
        "entry page must post the login form to the transaction login path"
    );

    EntryContext {
        cookie_name: cookie.name().to_owned(),
        cookie_value: cookie.value().to_owned(),
        csrf: html_attribute(&body, "data-csrf"),
        transaction_sqid,
    }
}

pub fn login_form(identifier: &str, password: &str, csrf: &str) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("identifier", identifier);
    serializer.append_pair("password", password);
    serializer.append_pair("csrf", csrf);
    serializer.finish()
}

pub fn login_path(fixture: &LoginFixture, transaction_sqid: &str) -> String {
    format!(
        "/oidc/{}/authorize/{transaction_sqid}/login",
        fixture.tenant_sqid
    )
}

pub async fn post_login(
    app: &TestApp,
    fixture: &LoginFixture,
    entry: &EntryContext,
    form: String,
) -> Response {
    app.client
        .post(app.url(&login_path(fixture, &entry.transaction_sqid)))
        .header("content-type", "application/x-www-form-urlencoded")
        .header(
            "cookie",
            format!("{}={}", entry.cookie_name, entry.cookie_value),
        )
        .body(form)
        .send()
        .await
        .expect("login request")
}

pub fn assert_html_headers(response: &Response) {
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert_eq!(
        response
            .headers()
            .get("pragma")
            .and_then(|value| value.to_str().ok()),
        Some("no-cache")
    );
    assert_eq!(
        response
            .headers()
            .get("referrer-policy")
            .and_then(|value| value.to_str().ok()),
        Some("no-referrer")
    );
    assert_eq!(
        response
            .headers()
            .get("x-content-type-options")
            .and_then(|value| value.to_str().ok()),
        Some("nosniff")
    );
    assert!(
        response
            .headers()
            .get("content-security-policy")
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value.contains("script-src 'none'")
                    && value.contains("form-action 'self'")
                    && value.contains("frame-ancestors 'none'")
            })
    );
    assert!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("text/html"))
    );
}

pub struct CodeRedirect {
    pub code: String,
    pub iss: String,
}

/// The RFC 6749 §4.1.2 authorization response: a 303 to the registered redirect URI carrying
/// `code`, `state`, and `iss`, with the transaction binding cookie cleared and no HTML body.
pub async fn assert_code_redirect(
    response: Response,
    fixture: &LoginFixture,
    entry: &EntryContext,
    state: &str,
) -> CodeRedirect {
    assert_eq!(response.status(), 303);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert_eq!(
        response
            .headers()
            .get("pragma")
            .and_then(|value| value.to_str().ok()),
        Some("no-cache")
    );
    assert_eq!(
        response
            .headers()
            .get("referrer-policy")
            .and_then(|value| value.to_str().ok()),
        Some("no-referrer")
    );
    assert_eq!(
        response
            .headers()
            .get("x-content-type-options")
            .and_then(|value| value.to_str().ok()),
        Some("nosniff")
    );

    let location = response
        .headers()
        .get("location")
        .expect("303 should carry a Location header")
        .to_str()
        .expect("Location should be ASCII")
        .to_owned();
    let set_cookie = response
        .headers()
        .get("set-cookie")
        .expect("303 should clear the binding cookie")
        .to_str()
        .expect("clearing cookie should be ASCII")
        .to_owned();
    let body = response.text().await.expect("read 303 body");
    assert!(body.is_empty(), "303 should not carry an HTML body");

    let query = location
        .strip_prefix(&fixture.redirect_uri)
        .and_then(|tail| tail.strip_prefix('?'))
        .expect("Location must be the registered redirect URI plus a query");
    let pairs: std::collections::HashMap<String, String> = form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();
    let code = pairs.get("code").expect("code parameter").to_owned();
    assert_eq!(code.len(), 22, "code is 16 random bytes base64url-encoded");
    assert!(
        code.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
        "code must be base64url URL_SAFE_NO_PAD"
    );
    assert_eq!(
        pairs.get("state").map(String::as_str),
        Some(state),
        "state must round-trip"
    );
    let iss = pairs.get("iss").expect("iss parameter").to_owned();
    assert!(
        iss.ends_with(&format!("/oidc/{}", fixture.tenant_sqid)),
        "iss must be the tenant issuer, got {iss}"
    );
    assert_eq!(pairs.len(), 3, "only code, state, and iss may be returned");
    assert!(!location.contains(&entry.csrf));
    assert!(!location.contains(&entry.transaction_sqid));
    assert!(!location.contains(USER_PASSWORD));
    assert!(!location.contains("private-nonce-sentinel"));

    let cookie = Cookie::parse(set_cookie)
        .expect("clearing cookie should parse")
        .into_owned();
    assert_eq!(cookie.name(), entry.cookie_name);
    assert_eq!(cookie.value(), "");
    assert_eq!(cookie.max_age(), Some(time::Duration::ZERO));
    let cookie_path = format!("/oidc/{}", fixture.tenant_sqid);
    assert_eq!(cookie.path(), Some(cookie_path.as_str()));

    CodeRedirect { code, iss }
}

/// Drives the full chain — create_user → entry → login → code → token exchange — and
/// returns the internal user id and the issued access token, so the UserInfo endpoint tests
/// can present a genuinely issued bearer token.
pub async fn drive_user_token(
    app: &TestApp,
    fixture: &LoginFixture,
    email: &str,
    state: &str,
) -> (Uuid, String) {
    let (user_id, body) = drive_user_token_response(app, fixture, email, state).await;
    let access_token = body["access_token"]
        .as_str()
        .expect("access token")
        .to_owned();
    (user_id, access_token)
}

/// Same drive chain as [`drive_user_token`], returning the raw token endpoint response body so
/// callers can inspect the issued `id_token` alongside the access token.
pub async fn drive_user_token_response(
    app: &TestApp,
    fixture: &LoginFixture,
    email: &str,
    state: &str,
) -> (Uuid, serde_json::Value) {
    let user_id = create_user(app, fixture, email, USER_PASSWORD).await;
    let entry = drive_entry(app, fixture, state).await;
    let response = post_login(
        app,
        fixture,
        &entry,
        login_form(email, USER_PASSWORD, &entry.csrf),
    )
    .await;
    let redirect = assert_code_redirect(response, fixture, &entry, state).await;

    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (name, value) in [
        ("grant_type", "authorization_code"),
        ("code", redirect.code.as_str()),
        ("redirect_uri", fixture.redirect_uri.as_str()),
        ("client_id", fixture.client_id.as_str()),
        ("code_verifier", CODE_VERIFIER),
    ] {
        serializer.append_pair(name, value);
    }
    let response = app
        .client
        .post(app.url(&format!("/oidc/{}/token", fixture.tenant_sqid)))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(serializer.finish())
        .send()
        .await
        .expect("token exchange request");
    assert_eq!(response.status(), 200, "token exchange should succeed");
    let body: serde_json::Value = response.json().await.expect("token response body");
    (user_id, body)
}

pub async fn create_user(
    app: &TestApp,
    fixture: &LoginFixture,
    email: &str,
    password: &str,
) -> Uuid {
    let user = app
        .api_create_user_with_credentials(
            &fixture.token,
            &fixture.tenant_sqid,
            &fixture.application_sqid,
            email,
            password,
        )
        .await;
    sqid_to_uuid(user["id"].as_str().expect("created user id"))
}

pub struct MfaUser {
    pub user_id: Uuid,
    pub email: String,
    pub code_generator: totp_rs::TOTP,
}

/// Seeds a subject with a TOTP-enabled credential written directly, so the application
/// credential cache never held a password-only vault (A1 fixture pattern).
pub async fn create_mfa_user(app: &TestApp, fixture: &LoginFixture, email: &str) -> MfaUser {
    let database = app.database().await;
    let user_id = Uuid::now_v7();
    let vault = CredentialVault::with_password(USER_PASSWORD.to_owned(), Argon2::default())
        .await
        .expect("build MFA fixture vault");
    let totp = Totp::generate("oceaniam-test", email).expect("generate TOTP");
    let provisioning_uri = totp.provisioning_uri();
    let vault = vault.enable_totp(totp.to_encrypted(&TEST_MASTER_KEY).expect("encrypt TOTP"));
    vault
        .write_to(user_id, &database)
        .await
        .expect("write MFA fixture credential");
    Users::create_user(
        user_id,
        fixture.application_id,
        CreateUserOpts {
            nickname: format!("mfa-user-{email}"),
            email: Some(email.to_owned()),
            phone: None,
        },
        &database,
    )
    .await
    .expect("create MFA fixture user");
    let code_generator =
        totp_rs::TOTP::from_url(&provisioning_uri).expect("parse the provisioning URI");
    MfaUser {
        user_id,
        email: email.to_owned(),
        code_generator,
    }
}

pub fn challenge_path(fixture: &LoginFixture, transaction_sqid: &str) -> String {
    format!(
        "/oidc/{}/authorize/{transaction_sqid}/challenge",
        fixture.tenant_sqid
    )
}

pub fn challenge_form(code: &str, csrf: &str) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("code", code);
    serializer.append_pair("csrf", csrf);
    serializer.finish()
}

pub async fn post_challenge(
    app: &TestApp,
    fixture: &LoginFixture,
    entry: &EntryContext,
    form: String,
) -> Response {
    app.client
        .post(app.url(&challenge_path(fixture, &entry.transaction_sqid)))
        .header("content-type", "application/x-www-form-urlencoded")
        .header(
            "cookie",
            format!("{}={}", entry.cookie_name, entry.cookie_value),
        )
        .body(form)
        .send()
        .await
        .expect("challenge request")
}

pub struct ChallengeFormPage {
    pub body: String,
    pub csrf: String,
}

fn challenge_form_csrf(body: &str) -> String {
    let marker = "name=\"csrf\" value=\"";
    let start = body
        .find(marker)
        .expect("challenge csrf input should exist")
        + marker.len();
    let end = body[start..]
        .find('"')
        .map(|offset| start + offset)
        .expect("challenge csrf value should close");
    body[start..end].to_owned()
}

pub async fn assert_challenge_form(
    response: Response,
    fixture: &LoginFixture,
    entry: &EntryContext,
    state: &str,
) -> ChallengeFormPage {
    assert_eq!(response.status(), 200);
    assert_html_headers(&response);
    assert!(response.headers().get("set-cookie").is_none());
    let body = response.text().await.expect("read challenge HTML");
    assert!(body.contains(&format!(
        "<form method=\"post\" action=\"/oidc/{}/authorize/{}/challenge\"",
        fixture.tenant_sqid, entry.transaction_sqid,
    )));
    assert!(body.contains("name=\"code\""));
    assert!(body.contains("inputmode=\"numeric\""));
    assert!(body.contains("autocomplete=\"one-time-code\""));
    let csrf = challenge_form_csrf(&body);
    assert_eq!(
        csrf, entry.csrf,
        "the challenge form must bind the original CSRF secret"
    );
    assert!(!body.contains("<script"));
    assert!(!body.contains(state));
    assert!(!body.contains("private-nonce-sentinel"));
    assert!(!body.contains(&fixture.redirect_uri));
    assert!(!body.contains(USER_PASSWORD));
    ChallengeFormPage { body, csrf }
}

/// Drives entry + password login for an MFA subject up to the rendered challenge form.
pub async fn drive_to_challenge_form(
    app: &TestApp,
    fixture: &LoginFixture,
    mfa_user: &MfaUser,
    state: &str,
) -> (EntryContext, ChallengeFormPage) {
    let entry = drive_entry(app, fixture, state).await;
    let response = post_login(
        app,
        fixture,
        &entry,
        login_form(&mfa_user.email, USER_PASSWORD, &entry.csrf),
    )
    .await;
    let page = assert_challenge_form(response, fixture, &entry, state).await;
    (entry, page)
}

impl MfaUser {
    pub fn current_code(&self) -> String {
        self.code_generator
            .generate_current()
            .expect("generate the current TOTP code")
    }
}
