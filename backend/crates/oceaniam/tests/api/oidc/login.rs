use argon2::Argon2;
use axum_extra::extract::cookie::Cookie;
use oceaniam_common::{config::BackendConfig, sqid::Sqid};
use oceaniam_credential::{CredentialVault, Totp};
use oceaniam_database::{
    helper::{
        applications::ApplicationHelper,
        oidc_clients::OidcClientsHelper,
        users::{CreateUserOpts, UserHelper},
    },
    model::{
        oidc_clients,
        prelude::{Audits, Challenges, OidcAuthorizationTransactions, Users},
        sea_orm_active_enums::{
            AuditType, ChallengeStatusType, OidcAuthorizationTransactionStatus,
        },
    },
};
use reqwest::Response;
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait, QueryFilter, Statement};
use url::form_urlencoded;
use uuid::Uuid;

use crate::support::{TestApp, spawn_app_with_isolated_schema_configured, sqid_to_uuid};

const CODE_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
const USER_PASSWORD: &str = "CorrectHorse42!";
// Deterministic non-zero test KEK, identical to the fixture key in `tests/support/mod.rs`.
const TEST_MASTER_KEY: [u8; 32] = [
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
];

struct LoginFixture {
    tenant_sqid: String,
    application_id: Uuid,
    application_sqid: String,
    oidc_client_id: Uuid,
    client_id: String,
    redirect_uri: String,
    token: String,
}

struct EntryContext {
    cookie_name: String,
    cookie_value: String,
    csrf: String,
    transaction_sqid: String,
}

async fn spawn_preview() -> TestApp {
    spawn_app_with_isolated_schema_configured(|config: &mut BackendConfig| {
        config.oidc.authorization_entry_preview_enabled = true;
    })
    .await
}

async fn seed_login_fixture(app: &TestApp) -> LoginFixture {
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

fn authorize_query(fixture: &LoginFixture, state: &str) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (name, value) in [
        ("response_type", "code"),
        ("client_id", fixture.client_id.as_str()),
        ("redirect_uri", fixture.redirect_uri.as_str()),
        ("scope", "openid"),
        ("state", state),
        ("nonce", "private-nonce-sentinel"),
        ("code_challenge", CODE_CHALLENGE),
        ("code_challenge_method", "S256"),
    ] {
        serializer.append_pair(name, value);
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

async fn drive_entry(app: &TestApp, fixture: &LoginFixture, state: &str) -> EntryContext {
    let response = app
        .client
        .get(app.url(&format!(
            "/oidc/{}/authorize?{}",
            fixture.tenant_sqid,
            authorize_query(fixture, state)
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

fn login_form(identifier: &str, password: &str, csrf: &str) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("identifier", identifier);
    serializer.append_pair("password", password);
    serializer.append_pair("csrf", csrf);
    serializer.finish()
}

fn login_path(fixture: &LoginFixture, transaction_sqid: &str) -> String {
    format!(
        "/oidc/{}/authorize/{transaction_sqid}/login",
        fixture.tenant_sqid
    )
}

async fn post_login(
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

fn assert_html_headers(response: &Response) {
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

async fn assert_success_page(response: Response, fixture: &LoginFixture, state: &str) -> String {
    assert_eq!(response.status(), 200);
    assert_html_headers(&response);
    assert!(response.headers().get("set-cookie").is_none());
    let body = response.text().await.expect("read success HTML");
    assert!(body.contains("authentication recorded"));
    assert!(body.contains("authorization continuation is not yet available"));
    assert!(!body.contains("<form"));
    assert!(!body.contains(state));
    assert!(!body.contains("private-nonce-sentinel"));
    assert!(!body.contains(&fixture.redirect_uri));
    body
}

async fn transaction_snapshot(
    app: &TestApp,
    entry: &EntryContext,
) -> oceaniam_database::model::oidc_authorization_transactions::Model {
    let database = app.database().await;
    OidcAuthorizationTransactions::find_by_id(sqid_to_uuid(&entry.transaction_sqid))
        .one(&database)
        .await
        .expect("read login transaction")
        .expect("transaction should remain stored")
}

async fn oidc_authenticate_audits(app: &TestApp) -> Vec<serde_json::Value> {
    let database = app.database().await;
    Audits::find()
        .filter(oceaniam_database::model::audits::Column::AuditType.eq(AuditType::OidcAuthenticate))
        .all(&database)
        .await
        .expect("list oidc_authenticate audits")
        .into_iter()
        .map(|audit| audit.payload)
        .collect()
}

async fn create_user(app: &TestApp, fixture: &LoginFixture, email: &str, password: &str) -> Uuid {
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

// NOTE: AI-generated test
#[tokio::test]
async fn entry_then_login_records_authentication_and_audit() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let user_id = create_user(&app, &fixture, "login-user@example.com", USER_PASSWORD).await;
    let state = "login-happy-path-state";
    let entry = drive_entry(&app, &fixture, state).await;

    let response = post_login(
        &app,
        &fixture,
        &entry,
        login_form("login-user@example.com", USER_PASSWORD, &entry.csrf),
    )
    .await;
    let body = assert_success_page(response, &fixture, state).await;
    assert!(!body.contains("login-user@example.com"));
    assert!(!body.contains(USER_PASSWORD));
    assert!(!body.contains(&entry.csrf));
    assert!(!body.contains(&entry.transaction_sqid));

    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(stored.subject_id, Some(user_id));
    assert_ne!(stored.subject_id, Some(stored.id));
    assert!(stored.authenticated_at.is_some());
    assert!(stored.authenticated_at.unwrap() >= stored.created_at);
    assert!(stored.authenticated_at.unwrap() < stored.expires_at);
    assert_eq!(stored.revision, 1);
    assert!(stored.terminal_at.is_none());

    let audits = oidc_authenticate_audits(&app).await;
    assert_eq!(audits.len(), 1, "success must audit exactly once");
    let audit = &audits[0];
    assert_eq!(audit["kind"], "oidc_authenticate");
    assert_eq!(
        audit["data"]["application_id"],
        fixture.application_id.to_string()
    );
    assert_eq!(audit["data"]["transaction_id"], stored.id.to_string());
    assert_eq!(audit["data"]["subject_id"], user_id.to_string());
    let audit_text = audit.to_string();
    assert!(!audit_text.contains(&fixture.redirect_uri));
    assert!(!audit_text.contains(state));
    assert!(!audit_text.contains("login-user@example.com"));
    assert!(!audit_text.contains(USER_PASSWORD));

    // The transaction is terminal for login purposes: a replay is indistinguishable from
    // an unknown transaction.
    let replay = post_login(
        &app,
        &fixture,
        &entry,
        login_form("login-user@example.com", USER_PASSWORD, &entry.csrf),
    )
    .await;
    assert_eq!(replay.status(), 404);
    assert_html_headers(&replay);
    let audits = oidc_authenticate_audits(&app).await;
    assert_eq!(audits.len(), 1, "a replay must not audit again");
}

// NOTE: AI-generated test
#[tokio::test]
async fn phone_identifier_login_records_authentication() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let response = app
        .client
        .post(app.url(&format!(
            "/tenants/{}/applications/{}/users",
            fixture.tenant_sqid, fixture.application_sqid
        )))
        .header("Authorization", format!("Bearer {}", fixture.token))
        .json(&serde_json::json!({
            "phone": "+8613912345678",
            "password": USER_PASSWORD,
        }))
        .send()
        .await
        .expect("create phone user request");
    assert_eq!(response.status(), 200, "phone user should be created");
    let user: serde_json::Value = response.json().await.expect("phone user response");
    let user_id = sqid_to_uuid(user["id"].as_str().expect("phone user id"));

    let entry = drive_entry(&app, &fixture, "phone-login-state").await;
    let response = post_login(
        &app,
        &fixture,
        &entry,
        login_form("+8613912345678", USER_PASSWORD, &entry.csrf),
    )
    .await;
    assert_success_page(response, &fixture, "phone-login-state").await;

    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(stored.subject_id, Some(user_id));
}

// NOTE: AI-generated test
#[tokio::test]
async fn credential_failures_are_indistinguishable_and_leave_no_trace() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let database = app.database().await;

    let active_user_id =
        create_user(&app, &fixture, "active-user@example.com", USER_PASSWORD).await;
    let inactive_user_id =
        create_user(&app, &fixture, "inactive-user@example.com", USER_PASSWORD).await;
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE subjects SET expires_at = clock_timestamp() - interval '1 minute' \
             WHERE id = $1",
            [inactive_user_id.into()],
        ))
        .await
        .expect("expire the inactive fixture subject");

    let mut failure_bodies = Vec::new();
    for (case, identifier, password) in [
        ("unknown user", "ghost@example.com", USER_PASSWORD),
        (
            "wrong password",
            "active-user@example.com",
            "WrongPassword9!",
        ),
        (
            "inactive subject",
            "inactive-user@example.com",
            USER_PASSWORD,
        ),
    ] {
        let entry = drive_entry(&app, &fixture, &format!("{case}-state")).await;
        let response = post_login(
            &app,
            &fixture,
            &entry,
            login_form(identifier, password, &entry.csrf),
        )
        .await;
        assert_eq!(response.status(), 401, "case: {case}");
        assert_html_headers(&response);
        assert!(response.headers().get("set-cookie").is_none());
        let body = response.text().await.expect("read failure HTML");
        assert!(
            body.contains(oceaniam_common::consts::USER_LOGIN_FAILED_MSG),
            "case: {case}"
        );
        assert!(!body.contains(identifier), "case: {case}");
        assert!(!body.contains(&entry.csrf), "case: {case}");
        failure_bodies.push(body);

        let stored = transaction_snapshot(&app, &entry).await;
        assert_eq!(
            stored.status,
            OidcAuthorizationTransactionStatus::Pending,
            "case: {case}"
        );
        assert_eq!(stored.revision, 0, "case: {case}");
        assert_eq!(stored.subject_id, None, "case: {case}");
        assert_eq!(stored.authenticated_at, None, "case: {case}");
    }

    for pair in failure_bodies.windows(2) {
        assert_eq!(
            pair[0], pair[1],
            "every credential failure must be byte-identical"
        );
    }
    assert!(
        oidc_authenticate_audits(&app).await.is_empty(),
        "failures must never audit"
    );

    let active = app
        .client
        .get(app.url(&format!(
            "/tenants/{}/applications/{}/users/{}",
            fixture.tenant_sqid,
            fixture.application_sqid,
            Sqid::from(active_user_id),
        )))
        .header("Authorization", format!("Bearer {}", fixture.token))
        .send()
        .await
        .expect("fixture sanity: active user still exists");
    assert_eq!(active.status(), 200);
}

// NOTE: AI-generated test
#[tokio::test]
async fn login_requires_intact_cookie_csrf_and_tenant_binding() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    create_user(&app, &fixture, "bound-user@example.com", USER_PASSWORD).await;

    let good = drive_entry(&app, &fixture, "binding-state").await;
    let other = drive_entry(&app, &fixture, "binding-other-state").await;
    let login_url = app.url(&login_path(&fixture, &good.transaction_sqid));

    let missing_cookie = app
        .client
        .post(&login_url)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(login_form(
            "bound-user@example.com",
            USER_PASSWORD,
            &good.csrf,
        ))
        .send()
        .await
        .expect("missing cookie request");
    assert_eq!(missing_cookie.status(), 404);
    let missing_cookie_body = missing_cookie.text().await.expect("read page");

    let wrong_transaction_cookie = app
        .client
        .post(&login_url)
        .header("content-type", "application/x-www-form-urlencoded")
        .header(
            "cookie",
            format!("{}={}", good.cookie_name, other.cookie_value),
        )
        .body(login_form(
            "bound-user@example.com",
            USER_PASSWORD,
            &good.csrf,
        ))
        .send()
        .await
        .expect("wrong transaction cookie request");
    assert_eq!(wrong_transaction_cookie.status(), 404);
    let wrong_cookie_body = wrong_transaction_cookie.text().await.expect("read page");
    assert_eq!(missing_cookie_body, wrong_cookie_body);

    let wrong_csrf = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    let wrong_csrf_response = post_login(
        &app,
        &fixture,
        &good,
        login_form("bound-user@example.com", USER_PASSWORD, wrong_csrf),
    )
    .await;
    assert_eq!(wrong_csrf_response.status(), 404);
    let wrong_csrf_body = wrong_csrf_response.text().await.expect("read page");
    assert_eq!(missing_cookie_body, wrong_csrf_body);

    let malformed_csrf_response = post_login(
        &app,
        &fixture,
        &good,
        login_form("bound-user@example.com", USER_PASSWORD, "not*base64url"),
    )
    .await;
    assert_eq!(malformed_csrf_response.status(), 404);
    let malformed_csrf_body = malformed_csrf_response.text().await.expect("read page");
    assert_eq!(missing_cookie_body, malformed_csrf_body);

    // The same transaction addressed through another tenant's namespace is indistinguishable
    // from an unknown transaction.
    let other_tenant = app.api_create_tenant(&fixture.token).await;
    let other_tenant_sqid = other_tenant["id"].as_str().unwrap().to_string();
    let cross_tenant = app
        .client
        .post(app.url(&format!(
            "/oidc/{}/authorize/{}/login",
            other_tenant_sqid, good.transaction_sqid
        )))
        .header("content-type", "application/x-www-form-urlencoded")
        .header(
            "cookie",
            format!("{}={}", good.cookie_name, good.cookie_value),
        )
        .body(login_form(
            "bound-user@example.com",
            USER_PASSWORD,
            &good.csrf,
        ))
        .send()
        .await
        .expect("cross-tenant login request");
    assert_eq!(cross_tenant.status(), 404);
    let cross_tenant_body = cross_tenant.text().await.expect("read page");
    assert_eq!(missing_cookie_body, cross_tenant_body);
    assert!(
        missing_cookie_body.contains("no longer available"),
        "the uniform page must not distinguish binding causes"
    );

    for entry in [&good, &other] {
        let stored = transaction_snapshot(&app, entry).await;
        assert_eq!(stored.status, OidcAuthorizationTransactionStatus::Pending);
        assert_eq!(stored.revision, 0);
        assert_eq!(stored.subject_id, None);
    }
    assert!(oidc_authenticate_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn login_rejects_expired_transaction_without_restoring_it() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    create_user(&app, &fixture, "expired-user@example.com", USER_PASSWORD).await;
    let entry = drive_entry(&app, &fixture, "expired-state").await;

    let database = app.database().await;
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE oidc_authorization_transactions \
             SET created_at = created_at - interval '11 minutes', \
                 expires_at = expires_at - interval '11 minutes' \
             WHERE id = $1",
            [sqid_to_uuid(&entry.transaction_sqid).into()],
        ))
        .await
        .expect("expire the transaction while preserving its lifetime");

    let response = post_login(
        &app,
        &fixture,
        &entry,
        login_form("expired-user@example.com", USER_PASSWORD, &entry.csrf),
    )
    .await;
    assert_eq!(response.status(), 404);
    assert_html_headers(&response);

    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(stored.status, OidcAuthorizationTransactionStatus::Pending);
    assert_eq!(stored.revision, 0);
    assert!(oidc_authenticate_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn concurrent_logins_have_exactly_one_winner() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let user_id = create_user(&app, &fixture, "race-user@example.com", USER_PASSWORD).await;
    let entry = drive_entry(&app, &fixture, "race-state").await;

    let login = || {
        app.client
            .post(app.url(&login_path(&fixture, &entry.transaction_sqid)))
            .header("content-type", "application/x-www-form-urlencoded")
            .header(
                "cookie",
                format!("{}={}", entry.cookie_name, entry.cookie_value),
            )
            .body(login_form(
                "race-user@example.com",
                USER_PASSWORD,
                &entry.csrf,
            ))
            .send()
    };
    let (first, second) = tokio::join!(login(), login());
    let first = first.expect("first concurrent login");
    let second = second.expect("second concurrent login");
    let mut statuses = [first.status().as_u16(), second.status().as_u16()];
    statuses.sort_unstable();
    assert_eq!(
        statuses,
        [200, 404],
        "exactly one concurrent login may authenticate the transaction"
    );

    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(stored.revision, 1);
    assert_eq!(stored.subject_id, Some(user_id));
    assert_eq!(
        oidc_authenticate_audits(&app).await.len(),
        1,
        "only the winning login may audit"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn login_revalidates_registration_and_policy_before_authenticating() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    create_user(
        &app,
        &fixture,
        "revalidation-user@example.com",
        USER_PASSWORD,
    )
    .await;
    let database = app.database().await;

    // Redirect registration removed between entry and login: live revalidation rejects.
    let redirect_gone = drive_entry(&app, &fixture, "redirect-gone-state").await;
    oceaniam_database::model::prelude::OidcClientRedirectUris::delete_many()
        .filter(
            oceaniam_database::model::oidc_client_redirect_uris::Column::OidcClientId
                .eq(fixture.oidc_client_id),
        )
        .exec(&database)
        .await
        .expect("remove redirect registration");
    let response = post_login(
        &app,
        &fixture,
        &redirect_gone,
        login_form(
            "revalidation-user@example.com",
            USER_PASSWORD,
            &redirect_gone.csrf,
        ),
    )
    .await;
    assert_eq!(response.status(), 400, "unregistered redirect must reject");
    assert!(response.headers().get("set-cookie").is_none());
    let stored = transaction_snapshot(&app, &redirect_gone).await;
    assert_eq!(stored.status, OidcAuthorizationTransactionStatus::Pending);

    // Client deleted between entry and login: the owner cascade removes the snapshot, so the
    // login is indistinguishable from an unknown transaction. A second client keeps the first
    // case's redirect deletion from interfering with this entry.
    let second_client_id = Uuid::now_v7();
    let second_client_public_id = format!("login-client-{}", second_client_id.simple());
    oidc_clients::Entity::create_client(
        second_client_id,
        fixture.application_id,
        second_client_public_id.clone(),
        "Login test client (delete case)".to_owned(),
        &database,
    )
    .await
    .expect("create second login test client");
    oidc_clients::Entity::create_redirect_uris(
        second_client_id,
        vec![fixture.redirect_uri.clone()],
        &database,
    )
    .await
    .expect("create second login test redirect");
    let second_fixture = LoginFixture {
        tenant_sqid: fixture.tenant_sqid.clone(),
        application_id: fixture.application_id,
        application_sqid: fixture.application_sqid.clone(),
        oidc_client_id: second_client_id,
        client_id: second_client_public_id,
        redirect_uri: fixture.redirect_uri.clone(),
        token: fixture.token.clone(),
    };
    let client_gone = drive_entry(&app, &second_fixture, "client-gone-state").await;
    let delete_client = app
        .client
        .delete(app.url(&format!(
            "/tenants/{}/applications/{}/oidc-clients/{}",
            fixture.tenant_sqid, fixture.application_sqid, second_fixture.client_id,
        )))
        .header("Authorization", format!("Bearer {}", fixture.token))
        .send()
        .await
        .expect("delete OIDC client request");
    assert!(delete_client.status().is_success());
    let response = post_login(
        &app,
        &second_fixture,
        &client_gone,
        login_form(
            "revalidation-user@example.com",
            USER_PASSWORD,
            &client_gone.csrf,
        ),
    )
    .await;
    assert_eq!(response.status(), 404);

    assert!(oidc_authenticate_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn login_revalidates_application_redirect_policy_before_authenticating() {
    let app = spawn_preview().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_sqid = tenant["id"].as_str().unwrap().to_string();
    let application = app.api_create_application(&token, &tenant_sqid).await;
    let application_sqid = application["application_id"].as_str().unwrap().to_string();
    let application_id = sqid_to_uuid(&application_sqid);
    let database = app.database().await;

    let mut configuration =
        oceaniam_database::config::application::ApplicationConfiguration::default();
    configuration.oidc.allow_insecure_loopback_redirect_uris = true;
    oceaniam_database::model::prelude::Applications::replace_configuration(
        application_id,
        configuration,
        &database,
    )
    .await
    .expect("enable loopback redirect policy");

    let oidc_client_id = Uuid::now_v7();
    let client_id = format!("login-policy-client-{}", oidc_client_id.simple());
    oidc_clients::Entity::create_client(
        oidc_client_id,
        application_id,
        client_id.clone(),
        "Login policy test client".to_owned(),
        &database,
    )
    .await
    .expect("create policy test client");
    oidc_clients::Entity::create_redirect_uris(
        oidc_client_id,
        vec!["http://localhost:3000/callback".to_owned()],
        &database,
    )
    .await
    .expect("create loopback redirect");
    let fixture = LoginFixture {
        tenant_sqid: tenant_sqid.clone(),
        application_id,
        application_sqid: application_sqid.clone(),
        oidc_client_id,
        client_id,
        redirect_uri: "http://localhost:3000/callback".to_owned(),
        token: token.clone(),
    };
    create_user(&app, &fixture, "policy-user@example.com", USER_PASSWORD).await;
    let entry = drive_entry(&app, &fixture, "policy-state").await;

    // Tighten the Application policy after the entry snapshot: the login must revalidate the
    // snapshot's redirect against the live configuration.
    oceaniam_database::model::prelude::Applications::replace_configuration(
        application_id,
        oceaniam_database::config::application::ApplicationConfiguration::default(),
        &database,
    )
    .await
    .expect("disable loopback redirect policy");

    let response = post_login(
        &app,
        &fixture,
        &entry,
        login_form("policy-user@example.com", USER_PASSWORD, &entry.csrf),
    )
    .await;
    assert_eq!(
        response.status(),
        400,
        "a redirect disallowed by the live policy must reject"
    );
    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(stored.status, OidcAuthorizationTransactionStatus::Pending);
    assert!(oidc_authenticate_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn login_transport_boundary_and_preview_gate() {
    let disabled = crate::support::spawn_app_with_isolated_schema().await;
    let tenant_sqid = Sqid::from(Uuid::now_v7()).to_string();
    let transaction_sqid = Sqid::from(Uuid::now_v7()).to_string();
    let disabled_response = disabled
        .client
        .post(disabled.url(&format!(
            "/oidc/{tenant_sqid}/authorize/{transaction_sqid}/login"
        )))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(vec![b'x'; 9 * 1024])
        .send()
        .await
        .expect("disabled preview login request");
    assert_eq!(
        disabled_response.status(),
        404,
        "the preview gate must precede body consumption"
    );

    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    create_user(&app, &fixture, "transport-user@example.com", USER_PASSWORD).await;
    let entry = drive_entry(&app, &fixture, "transport-state").await;
    let path = login_path(&fixture, &entry.transaction_sqid);

    let get = app
        .client
        .get(app.url(&path))
        .send()
        .await
        .expect("GET on login path");
    assert_eq!(get.status(), 405);

    let mixed_query = app
        .client
        .post(app.url(&format!("{path}?csrf=mixed")))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(login_form(
            "transport-user@example.com",
            USER_PASSWORD,
            &entry.csrf,
        ))
        .send()
        .await
        .expect("login POST with query");
    assert_eq!(mixed_query.status(), 400);

    let wrong_media = app
        .client
        .post(app.url(&path))
        .header("content-type", "application/json")
        .body("{}")
        .send()
        .await
        .expect("wrong media type login");
    assert_eq!(wrong_media.status(), 415);

    let oversized = app
        .client
        .post(app.url(&path))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(vec![b'x'; 8 * 1024 + 1])
        .send()
        .await
        .expect("oversized login body");
    assert_eq!(oversized.status(), 413);

    let duplicate = app
        .client
        .post(app.url(&path))
        .header("content-type", "application/x-www-form-urlencoded")
        .body("identifier=one&identifier=two")
        .send()
        .await
        .expect("duplicate field login");
    assert_eq!(duplicate.status(), 400);

    let invalid_tenant = app
        .client
        .post(app.url(&format!(
            "/oidc/@@invalid@@/authorize/{}/login",
            entry.transaction_sqid
        )))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(login_form(
            "transport-user@example.com",
            USER_PASSWORD,
            &entry.csrf,
        ))
        .send()
        .await
        .expect("invalid tenant sqid login");
    assert_eq!(invalid_tenant.status(), 400);

    let invalid_transaction = app
        .client
        .post(app.url(&format!(
            "/oidc/{}/authorize/@@invalid@@/login",
            fixture.tenant_sqid
        )))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(login_form(
            "transport-user@example.com",
            USER_PASSWORD,
            &entry.csrf,
        ))
        .send()
        .await
        .expect("invalid transaction sqid login");
    assert_eq!(invalid_transaction.status(), 400);

    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(stored.status, OidcAuthorizationTransactionStatus::Pending);
    assert_eq!(stored.revision, 0);
    assert!(oidc_authenticate_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[test]
fn openapi_documents_the_login_method() {
    let specification = oceaniam::app::build_openapi_spec();
    let path = specification
        .paths
        .paths
        .get("/oidc/{tenant_sqid}/authorize/{transaction_sqid}/login")
        .expect("login path should be documented");
    assert!(path.post.is_some());
    assert!(path.get.is_none());
}

// ---------------------------------------------------------------------------
// Slice A2: TOTP challenge (awaiting_challenge) flow
// ---------------------------------------------------------------------------

struct MfaUser {
    user_id: Uuid,
    email: String,
    code_generator: totp_rs::TOTP,
}

/// Seeds a subject with a TOTP-enabled credential written directly, so the application
/// credential cache never held a password-only vault (A1 fixture pattern).
async fn create_mfa_user(app: &TestApp, fixture: &LoginFixture, email: &str) -> MfaUser {
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

impl MfaUser {
    fn current_code(&self) -> String {
        self.code_generator
            .generate_current()
            .expect("generate the current TOTP code")
    }
}

fn challenge_path(fixture: &LoginFixture, transaction_sqid: &str) -> String {
    format!(
        "/oidc/{}/authorize/{transaction_sqid}/challenge",
        fixture.tenant_sqid
    )
}

fn challenge_form(code: &str, csrf: &str) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("code", code);
    serializer.append_pair("csrf", csrf);
    serializer.finish()
}

async fn post_challenge(
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

struct ChallengeFormPage {
    body: String,
    csrf: String,
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

async fn assert_challenge_form(
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

async fn challenge_snapshot(
    app: &TestApp,
    transaction: &oceaniam_database::model::oidc_authorization_transactions::Model,
) -> oceaniam_database::model::challenges::Model {
    let database = app.database().await;
    Challenges::find_by_id(
        transaction
            .challenge_id
            .expect("transaction must bind a challenge"),
    )
    .one(&database)
    .await
    .expect("read bound challenge")
    .expect("bound challenge should remain stored")
}

async fn create_challenge_audits(app: &TestApp) -> Vec<serde_json::Value> {
    let database = app.database().await;
    Audits::find()
        .filter(oceaniam_database::model::audits::Column::AuditType.eq(AuditType::CreateChallenge))
        .all(&database)
        .await
        .expect("list create_challenge audits")
        .into_iter()
        .map(|audit| audit.payload)
        .collect()
}

/// Drives entry + password login for an MFA subject up to the rendered challenge form.
async fn drive_to_challenge_form(
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

// NOTE: AI-generated test
#[tokio::test]
async fn mfa_login_creates_challenge_and_correct_code_authenticates() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let mfa_user = create_mfa_user(&app, &fixture, "mfa-happy@example.com").await;
    let state = "mfa-happy-state";
    let (entry, page) = drive_to_challenge_form(&app, &fixture, &mfa_user, state).await;
    assert!(!page.body.contains(&mfa_user.email));

    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );
    assert_eq!(stored.revision, 1);
    assert!(stored.challenge_id.is_some());
    assert_eq!(stored.subject_id, None);
    assert_eq!(stored.authenticated_at, None);
    assert!(stored.terminal_at.is_none());

    let challenge = challenge_snapshot(&app, &stored).await;
    assert_eq!(challenge.application_id, fixture.application_id);
    assert_eq!(challenge.subject_id, mfa_user.user_id);
    assert_eq!(challenge.status, ChallengeStatusType::Pending);
    assert_eq!(challenge.attempt_count, 0);
    assert_eq!(challenge.max_attempts, 5);
    let challenge_lifetime = challenge.expires_at - challenge.created_at;
    assert!(
        challenge_lifetime > chrono::Duration::seconds(299)
            && challenge_lifetime <= chrono::Duration::minutes(5),
        "the challenge lifetime must be five minutes, got {challenge_lifetime}"
    );
    assert!(challenge.expires_at < stored.expires_at);

    let create_audits = create_challenge_audits(&app).await;
    assert_eq!(
        create_audits.len(),
        1,
        "the challenge create must audit once"
    );
    let audit = &create_audits[0];
    assert_eq!(audit["kind"], "create_challenge");
    assert_eq!(audit["data"]["challenge_id"], challenge.id.to_string());
    assert_eq!(
        audit["data"]["application_id"],
        fixture.application_id.to_string()
    );
    assert_eq!(audit["data"]["subject_id"], mfa_user.user_id.to_string());
    assert_eq!(audit["data"]["factor_type"], "Totp");
    assert_eq!(audit["data"]["purpose"], "Signin");
    let audit_text = audit.to_string();
    assert!(!audit_text.contains(&fixture.redirect_uri));
    assert!(!audit_text.contains(state));
    assert!(!audit_text.contains(&mfa_user.email));
    assert!(oidc_authenticate_audits(&app).await.is_empty());

    let code = mfa_user.current_code();
    let response = post_challenge(&app, &fixture, &entry, challenge_form(&code, &page.csrf)).await;
    assert_success_page(response, &fixture, state).await;

    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(stored.revision, 2);
    assert_eq!(stored.subject_id, Some(mfa_user.user_id));
    assert!(stored.authenticated_at.is_some());
    assert_eq!(stored.challenge_id, Some(challenge.id));

    let challenge = challenge_snapshot(&app, &stored).await;
    assert_eq!(challenge.status, ChallengeStatusType::Consumed);
    assert!(challenge.consumed_at.is_some());
    assert_eq!(challenge.attempt_count, 0);

    let authenticate_audits = oidc_authenticate_audits(&app).await;
    assert_eq!(
        authenticate_audits.len(),
        1,
        "success must audit exactly once"
    );
    assert_eq!(
        authenticate_audits[0]["data"]["subject_id"],
        mfa_user.user_id.to_string()
    );
    assert_eq!(
        authenticate_audits[0]["data"]["transaction_id"],
        stored.id.to_string()
    );
    assert_eq!(
        create_challenge_audits(&app).await.len(),
        1,
        "no second challenge may be minted"
    );

    // The transaction is now terminal for the challenge leg: a replay is the uniform failure
    // page, never a state-machine reveal.
    let replay = post_challenge(&app, &fixture, &entry, challenge_form(&code, &page.csrf)).await;
    assert_eq!(replay.status(), 401);
    assert_html_headers(&replay);
    let replay_body = replay.text().await.expect("read replay page");
    assert!(replay_body.contains(oceaniam_common::consts::USER_LOGIN_FAILED_MSG));
    assert_eq!(oidc_authenticate_audits(&app).await.len(), 1);
}

// NOTE: AI-generated test
#[tokio::test]
async fn repeated_login_on_awaiting_challenge_rerenders_without_minting_a_challenge() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let mfa_user = create_mfa_user(&app, &fixture, "mfa-rerender@example.com").await;
    let (entry, first_page) =
        drive_to_challenge_form(&app, &fixture, &mfa_user, "mfa-rerender-state").await;

    let repeated = post_login(
        &app,
        &fixture,
        &entry,
        login_form(&mfa_user.email, USER_PASSWORD, &entry.csrf),
    )
    .await;
    let second_page = assert_challenge_form(repeated, &fixture, &entry, "mfa-rerender-state").await;
    assert_eq!(
        first_page.body, second_page.body,
        "a repeated login must re-render the identical challenge form"
    );

    let database = app.database().await;
    let challenge_count = Challenges::find()
        .all(&database)
        .await
        .expect("list challenges")
        .len();
    assert_eq!(
        challenge_count, 1,
        "a repeated login must not mint a new challenge"
    );
    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );
    assert_eq!(stored.revision, 1);
    assert_eq!(
        create_challenge_audits(&app).await.len(),
        1,
        "a repeated login must not audit another challenge create"
    );
    assert!(oidc_authenticate_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn wrong_code_increments_attempts_and_exhaustion_locks_the_challenge() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let mfa_user = create_mfa_user(&app, &fixture, "mfa-attempts@example.com").await;
    let (entry, page) =
        drive_to_challenge_form(&app, &fixture, &mfa_user, "mfa-attempts-state").await;

    // Reference failure page: a wrong-password login on a separate transaction.
    let reference_entry = drive_entry(&app, &fixture, "mfa-attempts-reference").await;
    let reference = post_login(
        &app,
        &fixture,
        &reference_entry,
        login_form(&mfa_user.email, "WrongPassword9!", &reference_entry.csrf),
    )
    .await;
    assert_eq!(reference.status(), 401);
    let reference_body = reference.text().await.expect("read reference failure page");

    let wrong_code = "000000";
    assert_ne!(
        wrong_code,
        mfa_user.current_code(),
        "fixture code must be wrong"
    );
    for attempt in 1..=5 {
        let response = post_challenge(
            &app,
            &fixture,
            &entry,
            challenge_form(wrong_code, &page.csrf),
        )
        .await;
        assert_eq!(response.status(), 401, "attempt {attempt}");
        assert_html_headers(&response);
        assert!(response.headers().get("set-cookie").is_none());
        let body = response.text().await.expect("read attempt failure page");
        assert_eq!(
            body, reference_body,
            "attempt {attempt} must match the login failure page"
        );
        assert!(!body.contains(&entry.transaction_sqid));

        let challenge = challenge_snapshot(&app, &transaction_snapshot(&app, &entry).await).await;
        assert_eq!(
            challenge.attempt_count, attempt,
            "attempt {attempt} must be counted"
        );
        assert_eq!(challenge.status, ChallengeStatusType::Pending);
    }

    // Exhausted: further attempts — even the correct code — are the same uniform failure and no
    // longer counted.
    for (case, code) in [
        ("sixth wrong code", wrong_code.to_owned()),
        ("correct code after exhaustion", mfa_user.current_code()),
    ] {
        let response =
            post_challenge(&app, &fixture, &entry, challenge_form(&code, &page.csrf)).await;
        assert_eq!(response.status(), 401, "case: {case}");
        let body = response.text().await.expect("read exhausted failure page");
        assert_eq!(body, reference_body, "case: {case}");
        let challenge = challenge_snapshot(&app, &transaction_snapshot(&app, &entry).await).await;
        assert_eq!(challenge.attempt_count, 5, "case: {case}");
        assert_eq!(
            challenge.status,
            ChallengeStatusType::Pending,
            "case: {case}"
        );
    }

    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );
    assert_eq!(stored.subject_id, None);
    assert!(oidc_authenticate_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn expired_challenge_is_rejected_and_rerender_becomes_unavailable() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let mfa_user = create_mfa_user(&app, &fixture, "mfa-expired@example.com").await;
    let (entry, page) =
        drive_to_challenge_form(&app, &fixture, &mfa_user, "mfa-expired-state").await;
    let challenge = challenge_snapshot(&app, &transaction_snapshot(&app, &entry).await).await;

    let database = app.database().await;
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE challenges SET expires_at = clock_timestamp() - interval '1 minute' \
             WHERE id = $1",
            [challenge.id.into()],
        ))
        .await
        .expect("expire the challenge");

    let response = post_challenge(
        &app,
        &fixture,
        &entry,
        challenge_form(&mfa_user.current_code(), &page.csrf),
    )
    .await;
    assert_eq!(response.status(), 401);
    let body = response.text().await.expect("read expired challenge page");
    assert!(body.contains(oceaniam_common::consts::USER_LOGIN_FAILED_MSG));

    let challenge = challenge_snapshot(&app, &transaction_snapshot(&app, &entry).await).await;
    assert_eq!(challenge.status, ChallengeStatusType::Pending);
    assert_eq!(challenge.attempt_count, 0);
    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );
    assert!(oidc_authenticate_audits(&app).await.is_empty());

    // The dead challenge no longer re-renders: a repeated login is the generic unavailable page.
    let repeated = post_login(
        &app,
        &fixture,
        &entry,
        login_form(&mfa_user.email, USER_PASSWORD, &entry.csrf),
    )
    .await;
    assert_eq!(repeated.status(), 404);
    let repeated_body = repeated.text().await.expect("read unavailable page");
    assert!(repeated_body.contains("no longer available"));
}

// NOTE: AI-generated test
#[tokio::test]
async fn replayed_code_is_rejected_by_anti_replay() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let mfa_user = create_mfa_user(&app, &fixture, "mfa-replay@example.com").await;

    let code = mfa_user.current_code();
    let (first_entry, first_page) =
        drive_to_challenge_form(&app, &fixture, &mfa_user, "mfa-replay-first").await;
    let first = post_challenge(
        &app,
        &fixture,
        &first_entry,
        challenge_form(&code, &first_page.csrf),
    )
    .await;
    assert_success_page(first, &fixture, "mfa-replay-first").await;

    // A second transaction for the same subject reusing the same one-time code is rejected.
    let (second_entry, second_page) =
        drive_to_challenge_form(&app, &fixture, &mfa_user, "mfa-replay-second").await;
    let replay = post_challenge(
        &app,
        &fixture,
        &second_entry,
        challenge_form(&code, &second_page.csrf),
    )
    .await;
    assert_eq!(replay.status(), 401);
    let replay_body = replay.text().await.expect("read replay failure page");
    assert!(replay_body.contains(oceaniam_common::consts::USER_LOGIN_FAILED_MSG));

    let second_challenge =
        challenge_snapshot(&app, &transaction_snapshot(&app, &second_entry).await).await;
    assert_eq!(second_challenge.status, ChallengeStatusType::Pending);
    assert_eq!(second_challenge.attempt_count, 1);
    let second_stored = transaction_snapshot(&app, &second_entry).await;
    assert_eq!(
        second_stored.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );
    assert_eq!(
        oidc_authenticate_audits(&app).await.len(),
        1,
        "only the first flow may audit"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn challenge_requires_intact_cookie_csrf_and_tenant_binding() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let mfa_user = create_mfa_user(&app, &fixture, "mfa-binding@example.com").await;
    let (entry, page) =
        drive_to_challenge_form(&app, &fixture, &mfa_user, "mfa-binding-state").await;
    let code = mfa_user.current_code();
    let challenge_url = app.url(&challenge_path(&fixture, &entry.transaction_sqid));

    // Reference unavailable page: a login POST without its cookie.
    let login_url = app.url(&login_path(&fixture, &entry.transaction_sqid));
    let login_missing_cookie = app
        .client
        .post(&login_url)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(login_form(&mfa_user.email, USER_PASSWORD, &entry.csrf))
        .send()
        .await
        .expect("login missing-cookie request");
    assert_eq!(login_missing_cookie.status(), 404);
    let reference_body = login_missing_cookie
        .text()
        .await
        .expect("read login unavailable page");

    let missing_cookie = app
        .client
        .post(&challenge_url)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(challenge_form(&code, &page.csrf))
        .send()
        .await
        .expect("challenge missing-cookie request");
    assert_eq!(missing_cookie.status(), 404);
    assert_eq!(
        missing_cookie.text().await.expect("read page"),
        reference_body
    );

    let wrong_csrf = post_challenge(
        &app,
        &fixture,
        &entry,
        challenge_form(&code, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
    )
    .await;
    assert_eq!(wrong_csrf.status(), 404);
    assert_eq!(wrong_csrf.text().await.expect("read page"), reference_body);

    let other = drive_entry(&app, &fixture, "mfa-binding-other-state").await;
    let wrong_transaction_cookie = app
        .client
        .post(&challenge_url)
        .header("content-type", "application/x-www-form-urlencoded")
        .header(
            "cookie",
            format!("{}={}", entry.cookie_name, other.cookie_value),
        )
        .body(challenge_form(&code, &page.csrf))
        .send()
        .await
        .expect("challenge wrong-cookie request");
    assert_eq!(wrong_transaction_cookie.status(), 404);
    assert_eq!(
        wrong_transaction_cookie.text().await.expect("read page"),
        reference_body
    );

    let other_tenant = app.api_create_tenant(&fixture.token).await;
    let other_tenant_sqid = other_tenant["id"].as_str().unwrap().to_string();
    let cross_tenant = app
        .client
        .post(app.url(&format!(
            "/oidc/{}/authorize/{}/challenge",
            other_tenant_sqid, entry.transaction_sqid
        )))
        .header("content-type", "application/x-www-form-urlencoded")
        .header(
            "cookie",
            format!("{}={}", entry.cookie_name, entry.cookie_value),
        )
        .body(challenge_form(&code, &page.csrf))
        .send()
        .await
        .expect("cross-tenant challenge request");
    assert_eq!(cross_tenant.status(), 404);
    assert_eq!(
        cross_tenant.text().await.expect("read page"),
        reference_body
    );

    let challenge = challenge_snapshot(&app, &transaction_snapshot(&app, &entry).await).await;
    assert_eq!(challenge.status, ChallengeStatusType::Pending);
    assert_eq!(challenge.attempt_count, 0);
    assert!(oidc_authenticate_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn challenge_on_pending_or_authenticated_transaction_is_indistinguishable_failure() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    create_user(&app, &fixture, "plain-user@example.com", USER_PASSWORD).await;

    // Pending transaction: entry only, no login yet. A wrong-password login on the same
    // transaction supplies the reference failure page.
    let pending_entry = drive_entry(&app, &fixture, "probe-pending-state").await;
    let reference = post_login(
        &app,
        &fixture,
        &pending_entry,
        login_form(
            "plain-user@example.com",
            "WrongPassword9!",
            &pending_entry.csrf,
        ),
    )
    .await;
    assert_eq!(reference.status(), 401);
    let reference_body = reference.text().await.expect("read reference failure page");

    let probed = post_challenge(
        &app,
        &fixture,
        &pending_entry,
        challenge_form("123456", &pending_entry.csrf),
    )
    .await;
    assert_eq!(probed.status(), 401);
    assert_html_headers(&probed);
    let probed_body = probed.text().await.expect("read probed failure page");
    assert_eq!(
        probed_body, reference_body,
        "a challenge POST on a pending transaction must be byte-identical to a credential failure"
    );
    let pending_stored = transaction_snapshot(&app, &pending_entry).await;
    assert_eq!(
        pending_stored.status,
        OidcAuthorizationTransactionStatus::Pending
    );
    assert_eq!(pending_stored.challenge_id, None);

    // Authenticated transaction (non-MFA direct login): same uniform failure, no state reveal.
    let authenticated_entry = drive_entry(&app, &fixture, "probe-authenticated-state").await;
    let success = post_login(
        &app,
        &fixture,
        &authenticated_entry,
        login_form(
            "plain-user@example.com",
            USER_PASSWORD,
            &authenticated_entry.csrf,
        ),
    )
    .await;
    assert_success_page(success, &fixture, "probe-authenticated-state").await;
    let probed = post_challenge(
        &app,
        &fixture,
        &authenticated_entry,
        challenge_form("123456", &authenticated_entry.csrf),
    )
    .await;
    assert_eq!(probed.status(), 401);
    assert_eq!(
        probed.text().await.expect("read page"),
        reference_body,
        "a challenge POST on an authenticated transaction must be byte-identical too"
    );

    let database = app.database().await;
    assert!(
        Challenges::find()
            .all(&database)
            .await
            .expect("list challenges")
            .is_empty(),
        "probing must never mint a challenge"
    );
    assert_eq!(
        oidc_authenticate_audits(&app).await.len(),
        1,
        "only the direct login may audit"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn concurrent_challenge_attempts_have_exactly_one_winner() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let mfa_user = create_mfa_user(&app, &fixture, "mfa-race@example.com").await;
    let (entry, page) = drive_to_challenge_form(&app, &fixture, &mfa_user, "mfa-race-state").await;
    let code = mfa_user.current_code();

    let attempt = || {
        app.client
            .post(app.url(&challenge_path(&fixture, &entry.transaction_sqid)))
            .header("content-type", "application/x-www-form-urlencoded")
            .header(
                "cookie",
                format!("{}={}", entry.cookie_name, entry.cookie_value),
            )
            .body(challenge_form(&code, &page.csrf))
            .send()
    };
    let (first, second) = tokio::join!(attempt(), attempt());
    let first = first.expect("first concurrent challenge");
    let second = second.expect("second concurrent challenge");
    let mut statuses = [first.status().as_u16(), second.status().as_u16()];
    statuses.sort_unstable();
    assert_eq!(
        statuses,
        [200, 401],
        "exactly one concurrent challenge attempt may authenticate the transaction"
    );

    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(stored.revision, 2);
    assert_eq!(stored.subject_id, Some(mfa_user.user_id));
    let challenge = challenge_snapshot(&app, &stored).await;
    assert_eq!(challenge.status, ChallengeStatusType::Consumed);
    assert_eq!(
        oidc_authenticate_audits(&app).await.len(),
        1,
        "only the winning challenge may audit"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn concurrent_login_and_challenge_never_mint_a_second_challenge() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let mfa_user = create_mfa_user(&app, &fixture, "mfa-interleave@example.com").await;
    let (entry, page) =
        drive_to_challenge_form(&app, &fixture, &mfa_user, "mfa-interleave-state").await;

    let relogin = || {
        app.client
            .post(app.url(&login_path(&fixture, &entry.transaction_sqid)))
            .header("content-type", "application/x-www-form-urlencoded")
            .header(
                "cookie",
                format!("{}={}", entry.cookie_name, entry.cookie_value),
            )
            .body(login_form(&mfa_user.email, USER_PASSWORD, &entry.csrf))
            .send()
    };
    let attempt = || {
        app.client
            .post(app.url(&challenge_path(&fixture, &entry.transaction_sqid)))
            .header("content-type", "application/x-www-form-urlencoded")
            .header(
                "cookie",
                format!("{}={}", entry.cookie_name, entry.cookie_value),
            )
            .body(challenge_form(&mfa_user.current_code(), &page.csrf))
            .send()
    };
    let (relogin, attempt) = tokio::join!(relogin(), attempt());
    let relogin = relogin.expect("concurrent repeated login");
    let attempt = attempt.expect("concurrent challenge");
    assert_eq!(
        attempt.status(),
        200,
        "the challenge must succeed exactly once"
    );
    assert!(
        relogin.status() == 200 || relogin.status() == 404,
        "a repeated login either re-renders or reports the transaction unavailable, got {}",
        relogin.status()
    );

    let database = app.database().await;
    assert_eq!(
        Challenges::find()
            .all(&database)
            .await
            .expect("list challenges")
            .len(),
        1,
        "no interleaving may mint a second challenge"
    );
    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(create_challenge_audits(&app).await.len(), 1);
    assert_eq!(oidc_authenticate_audits(&app).await.len(), 1);
}

// NOTE: AI-generated test
#[tokio::test]
async fn challenge_revalidates_registration_before_verifying() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let mfa_user = create_mfa_user(&app, &fixture, "mfa-revalidation@example.com").await;
    let database = app.database().await;

    // Redirect registration removed between login and challenge: live revalidation rejects and
    // the challenge is not consumed.
    let (redirect_gone, redirect_page) =
        drive_to_challenge_form(&app, &fixture, &mfa_user, "mfa-redirect-gone-state").await;
    oceaniam_database::model::prelude::OidcClientRedirectUris::delete_many()
        .filter(
            oceaniam_database::model::oidc_client_redirect_uris::Column::OidcClientId
                .eq(fixture.oidc_client_id),
        )
        .exec(&database)
        .await
        .expect("remove redirect registration");
    let response = post_challenge(
        &app,
        &fixture,
        &redirect_gone,
        challenge_form(&mfa_user.current_code(), &redirect_page.csrf),
    )
    .await;
    assert_eq!(response.status(), 400, "unregistered redirect must reject");
    assert!(response.headers().get("set-cookie").is_none());
    let challenge =
        challenge_snapshot(&app, &transaction_snapshot(&app, &redirect_gone).await).await;
    assert_eq!(challenge.status, ChallengeStatusType::Pending);
    assert_eq!(challenge.attempt_count, 0);
    let stored = transaction_snapshot(&app, &redirect_gone).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );

    // Client deleted between login and challenge: the owner cascade removes the snapshot, so the
    // challenge POST is indistinguishable from an unknown transaction. A second client keeps the
    // first case's redirect deletion from interfering with this entry.
    let second_client_id = Uuid::now_v7();
    let second_client_public_id = format!("login-client-{}", second_client_id.simple());
    oidc_clients::Entity::create_client(
        second_client_id,
        fixture.application_id,
        second_client_public_id.clone(),
        "Challenge test client (delete case)".to_owned(),
        &database,
    )
    .await
    .expect("create second challenge test client");
    oidc_clients::Entity::create_redirect_uris(
        second_client_id,
        vec![fixture.redirect_uri.clone()],
        &database,
    )
    .await
    .expect("create second challenge test redirect");
    let second_fixture = LoginFixture {
        tenant_sqid: fixture.tenant_sqid.clone(),
        application_id: fixture.application_id,
        application_sqid: fixture.application_sqid.clone(),
        oidc_client_id: second_client_id,
        client_id: second_client_public_id,
        redirect_uri: fixture.redirect_uri.clone(),
        token: fixture.token.clone(),
    };
    let (client_gone, client_page) =
        drive_to_challenge_form(&app, &second_fixture, &mfa_user, "mfa-client-gone-state").await;
    let delete_client = app
        .client
        .delete(app.url(&format!(
            "/tenants/{}/applications/{}/oidc-clients/{}",
            fixture.tenant_sqid, fixture.application_sqid, second_fixture.client_id,
        )))
        .header("Authorization", format!("Bearer {}", fixture.token))
        .send()
        .await
        .expect("delete OIDC client request");
    assert!(delete_client.status().is_success());
    let response = post_challenge(
        &app,
        &second_fixture,
        &client_gone,
        challenge_form(&mfa_user.current_code(), &client_page.csrf),
    )
    .await;
    assert_eq!(response.status(), 404);
    let body = response.text().await.expect("read unavailable page");
    assert!(body.contains("no longer available"));

    assert!(oidc_authenticate_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn challenge_rejects_expired_transaction_without_restoring_it() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let mfa_user = create_mfa_user(&app, &fixture, "mfa-tx-expired@example.com").await;
    let (entry, page) =
        drive_to_challenge_form(&app, &fixture, &mfa_user, "mfa-tx-expired-state").await;

    let database = app.database().await;
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE oidc_authorization_transactions \
             SET created_at = created_at - interval '11 minutes', \
                 expires_at = expires_at - interval '11 minutes' \
             WHERE id = $1",
            [sqid_to_uuid(&entry.transaction_sqid).into()],
        ))
        .await
        .expect("expire the transaction while preserving its lifetime");

    let response = post_challenge(
        &app,
        &fixture,
        &entry,
        challenge_form(&mfa_user.current_code(), &page.csrf),
    )
    .await;
    assert_eq!(response.status(), 404);
    assert_html_headers(&response);

    // A repeated login on the expired transaction is the same unavailable page.
    let repeated = post_login(
        &app,
        &fixture,
        &entry,
        login_form(&mfa_user.email, USER_PASSWORD, &entry.csrf),
    )
    .await;
    assert_eq!(repeated.status(), 404);

    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );
    assert_eq!(stored.revision, 1);
    let challenge = challenge_snapshot(&app, &stored).await;
    assert_eq!(challenge.status, ChallengeStatusType::Pending);
    assert!(oidc_authenticate_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[tokio::test]
async fn challenge_transport_boundary_and_preview_gate() {
    let disabled = crate::support::spawn_app_with_isolated_schema().await;
    let tenant_sqid = Sqid::from(Uuid::now_v7()).to_string();
    let transaction_sqid = Sqid::from(Uuid::now_v7()).to_string();
    let disabled_response = disabled
        .client
        .post(disabled.url(&format!(
            "/oidc/{tenant_sqid}/authorize/{transaction_sqid}/challenge"
        )))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(vec![b'x'; 9 * 1024])
        .send()
        .await
        .expect("disabled preview challenge request");
    assert_eq!(
        disabled_response.status(),
        404,
        "the preview gate must precede body consumption"
    );

    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let mfa_user = create_mfa_user(&app, &fixture, "mfa-transport@example.com").await;
    let (entry, page) =
        drive_to_challenge_form(&app, &fixture, &mfa_user, "mfa-transport-state").await;
    let path = challenge_path(&fixture, &entry.transaction_sqid);

    let get = app
        .client
        .get(app.url(&path))
        .send()
        .await
        .expect("GET on challenge path");
    assert_eq!(get.status(), 405);

    let mixed_query = app
        .client
        .post(app.url(&format!("{path}?csrf=mixed")))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(challenge_form("123456", &page.csrf))
        .send()
        .await
        .expect("challenge POST with query");
    assert_eq!(mixed_query.status(), 400);

    let wrong_media = app
        .client
        .post(app.url(&path))
        .header("content-type", "application/json")
        .body("{}")
        .send()
        .await
        .expect("wrong media type challenge");
    assert_eq!(wrong_media.status(), 415);

    let oversized = app
        .client
        .post(app.url(&path))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(vec![b'x'; 8 * 1024 + 1])
        .send()
        .await
        .expect("oversized challenge body");
    assert_eq!(oversized.status(), 413);

    let duplicate = app
        .client
        .post(app.url(&path))
        .header("content-type", "application/x-www-form-urlencoded")
        .body("code=one&code=two")
        .send()
        .await
        .expect("duplicate field challenge");
    assert_eq!(duplicate.status(), 400);

    let invalid_tenant = app
        .client
        .post(app.url(&format!(
            "/oidc/@@invalid@@/authorize/{}/challenge",
            entry.transaction_sqid
        )))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(challenge_form("123456", &page.csrf))
        .send()
        .await
        .expect("invalid tenant sqid challenge");
    assert_eq!(invalid_tenant.status(), 400);

    let invalid_transaction = app
        .client
        .post(app.url(&format!(
            "/oidc/{}/authorize/@@invalid@@/challenge",
            fixture.tenant_sqid
        )))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(challenge_form("123456", &page.csrf))
        .send()
        .await
        .expect("invalid transaction sqid challenge");
    assert_eq!(invalid_transaction.status(), 400);

    let stored = transaction_snapshot(&app, &entry).await;
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );
    assert_eq!(stored.revision, 1);
    assert!(oidc_authenticate_audits(&app).await.is_empty());
}

// NOTE: AI-generated test
#[test]
fn openapi_documents_the_challenge_method() {
    let specification = oceaniam::app::build_openapi_spec();
    let path = specification
        .paths
        .paths
        .get("/oidc/{tenant_sqid}/authorize/{transaction_sqid}/challenge")
        .expect("challenge path should be documented");
    assert!(path.post.is_some());
    assert!(path.get.is_none());
}
