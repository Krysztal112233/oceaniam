//! Integration tests for development accounts (see `docs/design/DEVELOPMENT_ACCOUNTS.md`).

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use oceaniam::state::{
    dev_account_expiry::{DevAccountExpirationRow, process_expiration_message},
    keybox::{EncodedJwt, SignJwtOptions},
};
use oceaniam_auth::jwt::Claim;
use oceaniam_common::sqid::Sqid;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::support::{TestApp, spawn_app_with_isolated_schema};

struct DevAccountFixture {
    tenant_id: String,
    application_id: String,
    secret_id: String,
    secret: String,
}

/// Creates a tenant, an application, and an application secret bound to it.
async fn setup_fixture(app: &TestApp, token: &str) -> DevAccountFixture {
    let tenant = app.api_create_tenant(token).await;
    let tenant_id = tenant["id"].as_str().unwrap().to_owned();
    let application = app.api_create_application(token, &tenant_id).await;
    let application_id = application["application_id"].as_str().unwrap().to_owned();

    let created: Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let secret_id = created["id"].as_str().unwrap().to_owned();
    let secret = created["secret"].as_str().unwrap().to_owned();

    let bind = app
        .client
        .post(app.url(&format!("/secrets/{secret_id}/bindings")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "application_id": application_id }))
        .send()
        .await
        .unwrap();
    assert!(bind.status().is_success());

    DevAccountFixture {
        tenant_id,
        application_id,
        secret_id,
        secret,
    }
}

async fn api_create_development_user(
    app: &TestApp,
    token: &str,
    fixture: &DevAccountFixture,
    email: &str,
    password: &str,
    ttl_seconds: Option<u64>,
) -> reqwest::Response {
    let development = ttl_seconds
        .map(|ttl_seconds| json!({ "ttl_seconds": ttl_seconds }))
        .unwrap_or_else(|| json!({}));
    let body = json!({
        "email": email,
        "password": password,
        "development": development,
    });

    app.client
        .post(app.url(&format!(
            "/tenants/{}/applications/{}/users",
            fixture.tenant_id, fixture.application_id
        )))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .expect("create dev account request failed")
}

async fn api_sign_in(
    app: &TestApp,
    fixture: &DevAccountFixture,
    email: &str,
    password: &str,
) -> reqwest::Response {
    app.client
        .post(app.url(&format!(
            "/tenants/{}/applications/{}/tokens",
            fixture.tenant_id, fixture.application_id
        )))
        .header("X-OceanIAM-Application-Secret", &fixture.secret)
        .json(&json!({ "email": email, "password": password }))
        .send()
        .await
        .expect("application signin request failed")
}

fn sqid_to_uuid(sqid: &str) -> Uuid {
    Uuid::try_from(sqid.parse::<Sqid>().expect("invalid sqid")).expect("invalid uuid")
}

fn jwt_subject(jwt: &str) -> Uuid {
    let payload = jwt
        .split('.')
        .nth(1)
        .expect("jwt payload should be present");
    let decoded = URL_SAFE_NO_PAD
        .decode(payload)
        .expect("jwt payload should be base64url");
    let claim: Value = serde_json::from_slice(&decoded).expect("jwt payload should be JSON");
    Uuid::parse_str(claim["sub"].as_str().expect("jwt sub should be present"))
        .expect("jwt sub should be a UUID")
}

/// Reads the queue row for a subject from `pgmq.q_dev_account_expiration`.
///
/// The queue is database-global (the `pgmq` schema is shared across test schemas), so rows are
/// filtered by `subject_id` to stay isolated from concurrently running tests.
async fn read_queue_row(app: &TestApp, subject_id: Uuid) -> DevAccountExpirationRow {
    let database = app.database().await;
    let row = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT msg_id, read_ct, message FROM pgmq.q_dev_account_expiration \
             WHERE message->>'subject_id' = $1",
            [subject_id.to_string().into()],
        ))
        .await
        .expect("queue query failed")
        .expect("expected a queue message for the dev account");

    DevAccountExpirationRow {
        msg_id: row.try_get("", "msg_id").unwrap(),
        read_ct: row.try_get("", "read_ct").unwrap(),
        message: row.try_get("", "message").unwrap(),
    }
}

async fn read_queue_delay_seconds(app: &TestApp, msg_id: i64) -> i64 {
    app.database()
        .await
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT FLOOR(EXTRACT(EPOCH FROM vt - enqueued_at))::bigint AS delay_seconds \
             FROM pgmq.q_dev_account_expiration WHERE msg_id = $1",
            [msg_id.into()],
        ))
        .await
        .unwrap()
        .expect("queue row should exist")
        .try_get("", "delay_seconds")
        .unwrap()
}

/// Scenario 1: creating a dev account works and the account can sign in.
// NOTE: AI-generated test
#[tokio::test]
async fn create_development_user_then_sign_in_works() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture = setup_fixture(&app, &token).await;

    let resp = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev@example.com",
        "DevPassword123!",
        Some(3600),
    )
    .await;
    assert_eq!(resp.status(), 200, "create dev account should return 200");

    let body: Value = resp.json().await.unwrap();
    let user_id = body["id"].as_str().expect("user id should be present");
    assert!(!user_id.is_empty(), "user id should not be empty");
    let internal_id = sqid_to_uuid(user_id);
    let oidc_sub = Uuid::parse_str(
        body["oidc_sub"]
            .as_str()
            .expect("oidc_sub should be present"),
    )
    .expect("oidc_sub should be a UUID");
    assert_ne!(oidc_sub, internal_id);
    assert_eq!(oidc_sub.get_version_num(), 7);
    assert_eq!(body["email"].as_str(), Some("dev@example.com"));
    assert!(
        body["expires_at"].as_str().is_some(),
        "expires_at should be present"
    );

    // The response `expires_at` must match the persisted column (within sub-second rounding:
    // the response carries nanosecond precision, timestamptz stores microseconds), and the
    // column must fall within [now+ttl-5s, now+ttl] (ttl=3600).
    let expires_at = body["expires_at"].as_str().unwrap().to_owned();
    let database = app.database().await;
    let check = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT \
                ABS(EXTRACT(EPOCH FROM subjects.expires_at - $1::timestamptz)) < 1 AS matches_response, \
                EXTRACT(EPOCH FROM subjects.expires_at - now())::bigint AS remaining_seconds, \
                users.oidc_sub \
             FROM subjects JOIN users ON users.id = subjects.id WHERE subjects.id = $2",
            [expires_at.into(), internal_id.into()],
        ))
        .await
        .unwrap()
        .expect("subject row should exist");
    assert!(
        check.try_get::<bool>("", "matches_response").unwrap(),
        "response expires_at should equal the persisted subjects.expires_at"
    );
    let remaining = check.try_get::<i64>("", "remaining_seconds").unwrap();
    assert!(
        (3595..=3600).contains(&remaining),
        "expires_at should be ~now()+ttl, got remaining={remaining}s"
    );
    assert_eq!(check.try_get::<Uuid>("", "oidc_sub").unwrap(), oidc_sub);

    let signin = api_sign_in(&app, &fixture, "dev@example.com", "DevPassword123!").await;
    assert_eq!(signin.status(), 200, "dev account sign-in should succeed");
    let signin_body: Value = signin.json().await.unwrap();
    let jwt = signin_body["jwt"]
        .as_str()
        .expect("sign-in should return a jwt");
    assert_eq!(jwt_subject(jwt), oidc_sub);
}

/// Scenario 2: forcing `expires_at` into the past makes sign-in reject with 401.
// NOTE: AI-generated test
#[tokio::test]
async fn expired_dev_account_sign_in_is_rejected() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture = setup_fixture(&app, &token).await;

    let resp = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev@example.com",
        "DevPassword123!",
        Some(3600),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let subject_id = sqid_to_uuid(body["id"].as_str().unwrap());

    // Sanity: the account can sign in before being expired.
    let signin = api_sign_in(&app, &fixture, "dev@example.com", "DevPassword123!").await;
    assert_eq!(signin.status(), 200);

    // Force the account into the expired state directly in the database.
    let database = app.database().await;
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE subjects SET expires_at = now() - interval '1 minute' WHERE id = $1",
            [subject_id.into()],
        ))
        .await
        .expect("failed to force expires_at into the past");

    let signin = api_sign_in(&app, &fixture, "dev@example.com", "DevPassword123!").await;
    assert_eq!(
        signin.status(),
        401,
        "sign-in for an expired dev account should be rejected with 401"
    );
}

/// Scenario 3: a delayed expiration message lands in `pgmq.q_dev_account_expiration` with the
/// right payload.
// NOTE: AI-generated test
#[tokio::test]
async fn dev_account_creation_enqueues_expiration_message() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture = setup_fixture(&app, &token).await;

    let resp = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev@example.com",
        "DevPassword123!",
        Some(3600),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let subject_id = sqid_to_uuid(body["id"].as_str().unwrap());
    let application_id = sqid_to_uuid(&fixture.application_id);

    let row = read_queue_row(&app, subject_id).await;
    assert_eq!(row.read_ct, 0, "message should not have been read yet");

    let message = &row.message;
    assert_eq!(
        message["subject_id"].as_str(),
        Some(subject_id.to_string().as_str())
    );
    assert_eq!(
        message["application_id"].as_str(),
        Some(application_id.to_string().as_str())
    );

    // The pgmq delay is the timer: `vt` must be ~`ttl_seconds` after `enqueued_at`.
    assert_eq!(
        read_queue_delay_seconds(&app, row.msg_id).await,
        3600,
        "message delay should equal ttl_seconds"
    );
}

/// Scenario 4: invoking the consumer's message-processing function directly deletes the
/// account; a second invocation is a no-op success.
// NOTE: AI-generated test
#[tokio::test]
async fn consumer_processes_expiration_message_idempotently() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture = setup_fixture(&app, &token).await;

    let resp = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev@example.com",
        "DevPassword123!",
        Some(3600),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let user_id = body["id"].as_str().unwrap().to_owned();
    let subject_id = sqid_to_uuid(&user_id);

    // Populate the caches via a sign-in so the deletion exercises the cache-eviction path.
    let signin = api_sign_in(&app, &fixture, "dev@example.com", "DevPassword123!").await;
    assert_eq!(signin.status(), 200);

    let row = read_queue_row(&app, subject_id).await;

    process_expiration_message(&app.state, &row)
        .await
        .expect("processing the expiration message should succeed");

    // The account is permanently deleted: sign-in fails and the user is gone.
    let signin = api_sign_in(&app, &fixture, "dev@example.com", "DevPassword123!").await;
    assert_eq!(
        signin.status(),
        401,
        "sign-in should fail after the dev account was deleted"
    );

    let get_resp = app
        .client
        .get(app.url(&format!(
            "/tenants/{}/applications/{}/users/{user_id}",
            fixture.tenant_id, fixture.application_id
        )))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        get_resp.status(),
        404,
        "deleted dev account should return 404"
    );

    // Second invocation is a no-op success (deleting an already-gone account is success).
    process_expiration_message(&app.state, &row)
        .await
        .expect("processing the same expiration message twice should be a no-op success");
}

/// Helper: POST a development user to the users endpoint with an application secret instead of
/// an admin bearer token.
async fn api_create_development_user_with_secret(
    app: &TestApp,
    secret: Option<&str>,
    tenant_id: &str,
    application_id: &str,
    body: Value,
) -> reqwest::Response {
    let request = app
        .client
        .post(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/users"
        )))
        .json(&body);
    let request = match secret {
        Some(secret) => request.header("X-OceanIAM-Application-Secret", secret),
        None => request,
    };
    request
        .send()
        .await
        .expect("create dev account request failed")
}

/// Helper: POST to the token-refresh endpoint with a bearer JWT and the application secret.
async fn api_refresh(app: &TestApp, fixture: &DevAccountFixture, jwt: &str) -> reqwest::Response {
    app.client
        .post(app.url(&format!(
            "/tenants/{}/applications/{}/tokens/refresh",
            fixture.tenant_id, fixture.application_id
        )))
        .header("Authorization", format!("Bearer {jwt}"))
        .header("X-OceanIAM-Application-Secret", &fixture.secret)
        .send()
        .await
        .expect("token refresh request failed")
}

/// Helper: force `subjects.expires_at` into the past for a subject.
async fn force_expired(app: &TestApp, subject_id: Uuid) {
    app.database()
        .await
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE subjects SET expires_at = now() - interval '1 minute' WHERE id = $1",
            [subject_id.into()],
        ))
        .await
        .expect("failed to force expires_at into the past");
}

/// Scenario 5: a NORMAL application user (expires_at IS NULL) is unaffected by the lazy
/// expiration check that now runs on every sign-in.
// NOTE: AI-generated test
#[tokio::test]
async fn normal_account_sign_in_unaffected() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture = setup_fixture(&app, &token).await;

    let user = app
        .api_create_user(&token, &fixture.tenant_id, &fixture.application_id)
        .await;
    let user_id = user["id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .expect("normal user should be created");
    assert!(
        user.get("expires_at").is_none(),
        "permanent-user creation response should omit expires_at"
    );

    let database = app.database().await;
    let state = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT s.expires_at IS NULL AS permanent, \
                    NOT EXISTS ( \
                        SELECT 1 FROM pgmq.q_dev_account_expiration q \
                        WHERE q.message->>'subject_id' = s.id::text \
                    ) AS has_no_expiration_message \
             FROM subjects s WHERE s.id = $1",
            [sqid_to_uuid(user_id).into()],
        ))
        .await
        .unwrap()
        .expect("normal subject row should exist");
    assert!(state.try_get::<bool>("", "permanent").unwrap());
    assert!(
        state
            .try_get::<bool>("", "has_no_expiration_message")
            .unwrap(),
        "permanent user must not enqueue a development-account expiration message"
    );

    let signin = api_sign_in(&app, &fixture, "test@example.com", "TestPassword123!").await;
    assert_eq!(
        signin.status(),
        200,
        "normal account (NULL expires_at) sign-in must be unaffected"
    );
    let signin_body: Value = signin.json().await.unwrap();
    assert!(signin_body["jwt"].as_str().is_some());
}

/// Application JWT subjects resolve through the persisted, application-scoped external subject.
// NOTE: AI-generated test
#[tokio::test]
async fn application_token_uses_oidc_sub_without_internal_id_fallback_or_cross_app_access() {
    let app = spawn_app_with_isolated_schema().await;
    let root_token = app.root_signin().await;
    let fixture = setup_fixture(&app, &root_token).await;
    let user = app
        .api_create_user(&root_token, &fixture.tenant_id, &fixture.application_id)
        .await;
    let internal_id = sqid_to_uuid(user["id"].as_str().unwrap());
    let oidc_sub = Uuid::parse_str(user["oidc_sub"].as_str().unwrap()).unwrap();
    assert_ne!(internal_id, oidc_sub);

    let signin = api_sign_in(&app, &fixture, "test@example.com", "TestPassword123!").await;
    assert_eq!(signin.status(), 200);
    let signin_body: Value = signin.json().await.unwrap();
    let jwt = signin_body["jwt"].as_str().unwrap().to_owned();
    assert_eq!(jwt_subject(&jwt), oidc_sub);

    let application_uuid = sqid_to_uuid(&fixture.application_id);
    let tenant_uuid = sqid_to_uuid(&fixture.tenant_id);
    let configuration = app
        .state
        .applications
        .get_configuration(application_uuid)
        .await
        .unwrap();
    let EncodedJwt { jwt: forged, .. } = app
        .state
        .keyboxes
        .clone()
        .sign_jwt::<Claim>(
            internal_id,
            SignJwtOptions {
                tenant_id: tenant_uuid,
                iss: configuration.auth.token.issuer.clone(),
                aud: configuration.auth.token.audience.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(
        api_refresh(&app, &fixture, &forged).await.status(),
        400,
        "a signed token using a new user's internal ID must not fall back to users.id"
    );

    let sibling = app
        .api_create_application(&root_token, &fixture.tenant_id)
        .await;
    let sibling_id = sibling["application_id"].as_str().unwrap();
    let bind = app
        .client
        .post(app.url(&format!("/secrets/{}/bindings", fixture.secret_id)))
        .header("Authorization", format!("Bearer {root_token}"))
        .json(&json!({ "application_id": sibling_id }))
        .send()
        .await
        .unwrap();
    assert!(bind.status().is_success());

    let cross_application = app
        .client
        .post(app.url(&format!(
            "/tenants/{}/applications/{sibling_id}/tokens/refresh",
            fixture.tenant_id
        )))
        .header("Authorization", format!("Bearer {jwt}"))
        .header("X-OceanIAM-Application-Secret", &fixture.secret)
        .send()
        .await
        .unwrap();
    assert_eq!(cross_application.status(), 400);

    let refreshed = api_refresh(&app, &fixture, &jwt).await;
    assert_eq!(refreshed.status(), 200);
    let refreshed_body: Value = refreshed.json().await.unwrap();
    let refreshed_jwt = refreshed_body["jwt"].as_str().unwrap().to_owned();
    assert_eq!(jwt_subject(&refreshed_jwt), oidc_sub);
    let signout = app
        .client
        .delete(app.url(&format!(
            "/tenants/{}/applications/{}/tokens",
            fixture.tenant_id, fixture.application_id
        )))
        .header("Authorization", format!("Bearer {refreshed_jwt}"))
        .header("X-OceanIAM-Application-Secret", &fixture.secret)
        .send()
        .await
        .unwrap();
    assert_eq!(signout.status(), 200);

    app.database()
        .await
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE users SET oidc_sub = id WHERE id = $1",
            [internal_id.into()],
        ))
        .await
        .unwrap();
    let EncodedJwt {
        jwt: backfilled_token,
        ..
    } = app
        .state
        .keyboxes
        .clone()
        .sign_jwt::<Claim>(
            internal_id,
            SignJwtOptions {
                tenant_id: tenant_uuid,
                iss: configuration.auth.token.issuer,
                aud: configuration.auth.token.audience,
            },
        )
        .await
        .unwrap();
    let backfilled_refresh = api_refresh(&app, &fixture, &backfilled_token).await;
    assert_eq!(
        backfilled_refresh.status(),
        200,
        "a pre-migration subject equal to users.id must retain token continuity"
    );
    let backfilled_body: Value = backfilled_refresh.json().await.unwrap();
    assert_eq!(
        jwt_subject(backfilled_body["jwt"].as_str().unwrap()),
        internal_id
    );
}

/// Scenario 6: token refresh succeeds before expiry and is rejected with 401 after the account
/// is expired (lazy rejection on the refresh path, checked before the old token is revoked).
// NOTE: AI-generated test
#[tokio::test]
async fn expired_dev_account_refresh_is_rejected() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture = setup_fixture(&app, &token).await;

    let resp = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev@example.com",
        "DevPassword123!",
        Some(3600),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let subject_id = sqid_to_uuid(body["id"].as_str().unwrap());

    let signin = api_sign_in(&app, &fixture, "dev@example.com", "DevPassword123!").await;
    assert_eq!(signin.status(), 200);
    let signin_body: Value = signin.json().await.unwrap();
    let jwt = signin_body["jwt"].as_str().expect("jwt should be present");

    // Before expiry the refresh succeeds and returns a new jwt (the old one is revoked).
    let refresh = api_refresh(&app, &fixture, jwt).await;
    assert_eq!(
        refresh.status(),
        200,
        "refresh before expiry should succeed"
    );
    let refresh_body: Value = refresh.json().await.unwrap();
    let jwt2 = refresh_body["jwt"]
        .as_str()
        .expect("refresh should return a new jwt")
        .to_owned();

    force_expired(&app, subject_id).await;

    let refresh = api_refresh(&app, &fixture, &jwt2).await;
    assert_eq!(
        refresh.status(),
        401,
        "refresh for an expired dev account should be rejected with 401"
    );
}

/// Scenario 7: negative authorization — an application secret may only create dev accounts for
/// the application it is bound to, and an unauthenticated call is rejected.
// NOTE: AI-generated test
#[tokio::test]
async fn dev_accounts_negative_authorization() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture_a = setup_fixture(&app, &token).await;

    // A second application in the same tenant, with its own path.
    let application_b = app
        .api_create_application(&token, &fixture_a.tenant_id)
        .await;
    let application_b_id = application_b["application_id"].as_str().unwrap().to_owned();

    let body = json!({
        "email": "dev@example.com",
        "password": "DevPassword123!",
        "development": {},
    });

    // (a) App A's secret against app B's path -> 403.
    let resp = api_create_development_user_with_secret(
        &app,
        Some(&fixture_a.secret),
        &fixture_a.tenant_id,
        &application_b_id,
        body.clone(),
    )
    .await;
    assert_eq!(
        resp.status(),
        403,
        "a secret bound to app A must not create dev accounts for app B"
    );

    // (b) No auth header at all -> 401.
    let resp = api_create_development_user_with_secret(
        &app,
        None,
        &fixture_a.tenant_id,
        &fixture_a.application_id,
        body,
    )
    .await;
    assert_eq!(
        resp.status(),
        401,
        "unauthenticated dev account creation should be rejected"
    );
}

/// Scenario 8: an application-secret caller (not an admin) can self-serve dev accounts for its
/// own application.
// NOTE: AI-generated test
#[tokio::test]
async fn app_secret_caller_creates_dev_account() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture = setup_fixture(&app, &token).await;

    let resp = api_create_development_user_with_secret(
        &app,
        Some(&fixture.secret),
        &fixture.tenant_id,
        &fixture.application_id,
        json!({
            "email": "dev@example.com",
            "password": "DevPassword123!",
            "development": { "ttl_seconds": 3600 },
        }),
    )
    .await;
    assert_eq!(
        resp.status(),
        200,
        "app-secret caller should be able to create dev accounts for its own application"
    );
    let body: Value = resp.json().await.unwrap();
    assert!(body["expires_at"].as_str().is_some());

    let signin = api_sign_in(&app, &fixture, "dev@example.com", "DevPassword123!").await;
    assert_eq!(signin.status(), 200);
}

/// Scenario 9: nested ttl_seconds validation and semantics — invalid bounds are rejected,
/// `development: {}` defaults to 3600, and a custom TTL controls both expiration and queue delay.
// NOTE: AI-generated test
#[tokio::test]
async fn dev_account_ttl_validation_default_and_custom_value() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture = setup_fixture(&app, &token).await;

    let zero = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev-zero@example.com",
        "DevPassword123!",
        Some(0),
    )
    .await;
    assert_eq!(zero.status(), 400, "ttl_seconds=0 should be rejected");

    let overflow = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev-overflow@example.com",
        "DevPassword123!",
        Some(i32::MAX as u64 + 1),
    )
    .await;
    assert_eq!(
        overflow.status(),
        400,
        "ttl_seconds above the pgmq integer limit should be rejected"
    );

    let default = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev-default@example.com",
        "DevPassword123!",
        None,
    )
    .await;
    assert_eq!(default.status(), 200);
    let default_body: Value = default.json().await.unwrap();
    let default_subject_id = sqid_to_uuid(default_body["id"].as_str().unwrap());
    let default_row = read_queue_row(&app, default_subject_id).await;
    assert_eq!(
        read_queue_delay_seconds(&app, default_row.msg_id).await,
        3600,
        "development: {{}} should default ttl_seconds to 3600"
    );

    let custom = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev-custom@example.com",
        "DevPassword123!",
        Some(60),
    )
    .await;
    assert_eq!(custom.status(), 200);
    let custom_body: Value = custom.json().await.unwrap();
    let custom_subject_id = sqid_to_uuid(custom_body["id"].as_str().unwrap());
    let custom_expires_at = custom_body["expires_at"]
        .as_str()
        .expect("custom-TTL response should include expires_at");
    let custom_row = read_queue_row(&app, custom_subject_id).await;
    assert_eq!(
        read_queue_delay_seconds(&app, custom_row.msg_id).await,
        60,
        "custom ttl_seconds should control the pgmq delay"
    );

    let remaining = app
        .database()
        .await
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT EXTRACT(EPOCH FROM expires_at - now())::bigint AS remaining_seconds, \
                    ABS(EXTRACT(EPOCH FROM expires_at - $1::timestamptz)) < 1 AS matches_response \
             FROM subjects WHERE id = $2",
            [custom_expires_at.into(), custom_subject_id.into()],
        ))
        .await
        .unwrap()
        .expect("custom-TTL subject should exist");
    assert!(
        remaining.try_get::<bool>("", "matches_response").unwrap(),
        "custom-TTL response should match subjects.expires_at"
    );
    let remaining_seconds = remaining.try_get::<i64>("", "remaining_seconds").unwrap();
    assert!(
        (55..=60).contains(&remaining_seconds),
        "custom expires_at should be ~now()+60s, got {remaining_seconds}s"
    );
}

/// Patches the application's `development_accounts` configuration as the root admin.
async fn api_patch_development_accounts_config(
    app: &TestApp,
    token: &str,
    fixture: &DevAccountFixture,
    patch: Value,
) -> reqwest::Response {
    app.client
        .patch(app.url(&format!(
            "/tenants/{}/applications/{}/configuration",
            fixture.tenant_id, fixture.application_id
        )))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "development_accounts": patch }))
        .send()
        .await
        .expect("patch development_accounts configuration request failed")
}

/// Reads the application's `development_accounts` configuration as the root admin.
async fn api_get_development_accounts_config(
    app: &TestApp,
    token: &str,
    fixture: &DevAccountFixture,
) -> Value {
    let resp = app
        .client
        .get(app.url(&format!(
            "/tenants/{}/applications/{}/configuration",
            fixture.tenant_id, fixture.application_id
        )))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get configuration request failed");
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    body["configuration"]["development_accounts"].clone()
}

/// Scenario 10: `enabled=false` rejects development-account creation with 403 while permanent
/// user creation keeps working; re-enabling restores development-account creation.
// NOTE: AI-generated test
#[tokio::test]
async fn dev_account_disabled_configuration_rejects_creation() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture = setup_fixture(&app, &token).await;

    let disable =
        api_patch_development_accounts_config(&app, &token, &fixture, json!({ "enabled": false }))
            .await;
    assert_eq!(disable.status(), 200, "disabling should succeed");
    let config = api_get_development_accounts_config(&app, &token, &fixture).await;
    assert_eq!(config["enabled"], false);

    let rejected = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev-disabled@example.com",
        "DevPassword123!",
        None,
    )
    .await;
    assert_eq!(
        rejected.status(),
        403,
        "development-account creation should be rejected when disabled"
    );

    // The enabled check precedes TTL validation: even an invalid TTL gets 403.
    let rejected_invalid_ttl = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev-disabled-zero@example.com",
        "DevPassword123!",
        Some(0),
    )
    .await;
    assert_eq!(
        rejected_invalid_ttl.status(),
        403,
        "disabled applications should fail with 403 regardless of the requested TTL"
    );

    // Permanent user creation is unaffected by the development-account switch.
    let permanent = app
        .client
        .post(app.url(&format!(
            "/tenants/{}/applications/{}/users",
            fixture.tenant_id, fixture.application_id
        )))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "email": "permanent@example.com",
            "password": "PermanentPassword123!",
        }))
        .send()
        .await
        .expect("create permanent user request failed");
    assert_eq!(
        permanent.status(),
        200,
        "permanent user creation should still work when development accounts are disabled"
    );

    let enable =
        api_patch_development_accounts_config(&app, &token, &fixture, json!({ "enabled": true }))
            .await;
    assert_eq!(enable.status(), 200, "re-enabling should succeed");

    let accepted = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev-enabled@example.com",
        "DevPassword123!",
        None,
    )
    .await;
    assert_eq!(
        accepted.status(),
        200,
        "development-account creation should work after re-enabling"
    );
}

/// Scenario 11: custom `default_ttl_seconds` replaces the 3600-second default and
/// `max_ttl_seconds` caps requested TTLs.
// NOTE: AI-generated test
#[tokio::test]
async fn dev_account_custom_default_and_max_ttl() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture = setup_fixture(&app, &token).await;

    let patch = api_patch_development_accounts_config(
        &app,
        &token,
        &fixture,
        json!({ "default_ttl_seconds": 120, "max_ttl_seconds": 300 }),
    )
    .await;
    assert_eq!(patch.status(), 200);
    let config = api_get_development_accounts_config(&app, &token, &fixture).await;
    assert_eq!(config["default_ttl_seconds"], 120);
    assert_eq!(config["max_ttl_seconds"], 300);

    let above_max = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev-above-max@example.com",
        "DevPassword123!",
        Some(301),
    )
    .await;
    assert_eq!(
        above_max.status(),
        400,
        "ttl_seconds above max_ttl_seconds should be rejected"
    );

    let defaulted = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev-custom-default@example.com",
        "DevPassword123!",
        None,
    )
    .await;
    assert_eq!(defaulted.status(), 200);
    let defaulted_body: Value = defaulted.json().await.unwrap();
    let defaulted_subject_id = sqid_to_uuid(defaulted_body["id"].as_str().unwrap());
    let defaulted_row = read_queue_row(&app, defaulted_subject_id).await;
    assert_eq!(
        read_queue_delay_seconds(&app, defaulted_row.msg_id).await,
        120,
        "development: {{}} should use the configured default_ttl_seconds"
    );

    let at_max = api_create_development_user(
        &app,
        &token,
        &fixture,
        "dev-at-max@example.com",
        "DevPassword123!",
        Some(300),
    )
    .await;
    assert_eq!(
        at_max.status(),
        200,
        "ttl_seconds equal to max_ttl_seconds should be accepted"
    );
}

/// Scenario 12: semantically invalid `development_accounts` patches are rejected with 400
/// and leave the stored configuration unchanged.
// NOTE: AI-generated test
#[tokio::test]
async fn dev_account_invalid_configuration_patch_rejected() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let fixture = setup_fixture(&app, &token).await;

    let before = api_get_development_accounts_config(&app, &token, &fixture).await;

    for patch in [
        json!({ "default_ttl_seconds": 0 }),
        json!({ "max_ttl_seconds": 0 }),
        json!({ "default_ttl_seconds": 7200, "max_ttl_seconds": 3600 }),
        json!({ "max_ttl_seconds": i32::MAX as u64 + 1 }),
    ] {
        let resp = api_patch_development_accounts_config(&app, &token, &fixture, patch).await;
        assert_eq!(
            resp.status(),
            400,
            "invalid development_accounts patch should be rejected"
        );
    }

    let after = api_get_development_accounts_config(&app, &token, &fixture).await;
    assert_eq!(
        before, after,
        "rejected patches should not modify the stored configuration"
    );
}
