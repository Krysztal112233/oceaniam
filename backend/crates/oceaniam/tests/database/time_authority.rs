//! Coverage for the single-time-authority rule (Rule 3): timestamps persisted to the database
//! or evaluated by database predicates originate from the database clock (`clock_timestamp()`).

use chrono::{DateTime, Duration, FixedOffset};
use oceaniam_database::error::Error as DbError;
use oceaniam_database::helper::applications::ApplicationHelper;
use oceaniam_database::helper::audit_summary_by_application::AuditSummaryByApplicationHelper;
use oceaniam_database::helper::challenges::{ChallengesHelper, CreateChallengeOpts};
use oceaniam_database::helper::credentials::CredentialsHelper;
use oceaniam_database::helper::subjects::SubjectsHelper;
use oceaniam_database::helper::tenants::TenantsHelper;
use oceaniam_database::helper::users::{CreateUserOpts, UserHelper};
use oceaniam_database::helper::{db_clock_now, trend};
use oceaniam_database::model::sea_orm_active_enums::AuditType;
use oceaniam_database::model::{
    audits,
    prelude::{
        Applications, AuditSummaryByApplication, Audits, Challenges, Credentials, Subjects,
        Tenants, Users,
    },
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use serde_json::json;
use uuid::Uuid;

use crate::support::spawn_app_with_isolated_schema;

/// Asserts `actual` lies between the bracketing database-clock samples (µs resolution).
fn assert_within_db_samples(
    actual: DateTime<FixedOffset>,
    before: DateTime<FixedOffset>,
    after: DateTime<FixedOffset>,
    context: &str,
) {
    let actual = actual.timestamp_micros();
    assert!(
        before.timestamp_micros() <= actual && actual <= after.timestamp_micros(),
        "{context}: {actual} not within [{}, {}]",
        before.timestamp_micros(),
        after.timestamp_micros()
    );
}

/// Seeds the credential + subject + user chain (in that FK order) for an application.
async fn seed_user(database: &DatabaseConnection, application_id: Uuid) -> Uuid {
    let user_id = Uuid::now_v7();
    Credentials::upsert_credential(user_id, "phc-test".to_owned(), None, database)
        .await
        .expect("credential seed should succeed");
    Users::create_user(
        user_id,
        application_id,
        CreateUserOpts {
            nickname: "time-authority-test".to_owned(),
            email: Some(format!("{user_id}@example.com")),
            phone: None,
        },
        database,
    )
    .await
    .expect("user seed should succeed");
    user_id
}

// NOTE: AI-generated test
#[tokio::test]
async fn create_challenge_anchors_both_timestamps_on_one_db_sample() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;
    let subject_id = seed_user(&database, application_id).await;

    let ttl = Duration::milliseconds(1500);
    let before = db_clock_now(&database)
        .await
        .expect("clock sample should succeed");
    let challenge = Challenges::create_challenge(
        application_id,
        subject_id,
        CreateChallengeOpts::new().expires_after(ttl),
        &database,
    )
    .await
    .expect("challenge creation should succeed");
    let after = db_clock_now(&database)
        .await
        .expect("clock sample should succeed");

    // The row invariant holds exactly: both timestamps derive from a single sample.
    assert_eq!(
        challenge.expires_at - challenge.created_at,
        ttl,
        "expires_at - created_at must equal the ttl exactly"
    );

    // Both timestamps fall between the bracketing samples.
    assert_within_db_samples(challenge.created_at, before, after, "challenges.created_at");
    assert_within_db_samples(
        challenge.expires_at,
        before + ttl,
        after + ttl,
        "challenges.expires_at",
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn set_subject_expiration_negative_ttl_expires_immediately() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;
    let subject_id = seed_user(&database, application_id).await;

    let ttl = Duration::seconds(-5);
    let before = db_clock_now(&database)
        .await
        .expect("clock sample should succeed");
    let subject = Subjects::set_subject_expiration(subject_id, ttl, &database)
        .await
        .expect("stamping an existing subject should succeed");
    let after = db_clock_now(&database)
        .await
        .expect("clock sample should succeed");

    // A negative ttl is legal and stamps an already-past expiry on the database clock.
    let expires_at = subject
        .expires_at
        .expect("set_subject_expiration always stamps expires_at");
    assert_within_db_samples(expires_at, before + ttl, after + ttl, "subjects.expires_at");
    assert!(
        expires_at <= after,
        "negative ttl must produce an already-expired subject"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn ensure_subject_active_rejects_expired_and_allows_active_subjects() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    // A normal account (expires_at IS NULL) always passes.
    let plain = seed_user(&database, application_id).await;
    Subjects::ensure_subject_active(plain, &database)
        .await
        .expect("subject without expires_at should pass");

    // An unexpired time-limited account passes.
    let active = seed_user(&database, application_id).await;
    Subjects::set_subject_expiration(active, Duration::hours(1), &database)
        .await
        .expect("stamping should succeed");
    Subjects::ensure_subject_active(active, &database)
        .await
        .expect("unexpired subject should pass");

    // An expired account is rejected with the generic 401 login failure.
    let expired = seed_user(&database, application_id).await;
    Subjects::set_subject_expiration(expired, Duration::seconds(-1), &database)
        .await
        .expect("stamping should succeed");
    let error = Subjects::ensure_subject_active(expired, &database)
        .await
        .expect_err("expired subject should be rejected");
    match error {
        DbError::CustomMessage { code, msg, .. } => {
            assert_eq!(code, 401, "expired subject should map to 401, got {code}");
            assert_eq!(
                msg,
                oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
                "the message must not leak account existence"
            );
        }
        other => panic!("expected a 401 CustomMessage, got {other:?}"),
    }

    // A missing subject is indistinguishable from an expired one.
    let error = Subjects::ensure_subject_active(Uuid::now_v7(), &database)
        .await
        .expect_err("missing subject should be rejected");
    match error {
        DbError::CustomMessage { code, .. } => {
            assert_eq!(code, 401, "missing subject should map to 401, got {code}")
        }
        other => panic!("expected a 401 CustomMessage, got {other:?}"),
    }
}

// NOTE: AI-generated test
#[tokio::test]
async fn created_at_columns_come_from_the_database_clock() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;

    // tenants.created_at
    let tenant_id = Uuid::now_v7();
    let before = db_clock_now(&database)
        .await
        .expect("clock sample should succeed");
    let tenant = Tenants::create_tenant(tenant_id, None::<String>, &database)
        .await
        .expect("tenant creation should succeed");
    let after = db_clock_now(&database)
        .await
        .expect("clock sample should succeed");
    assert_within_db_samples(tenant.created_at, before, after, "tenants.created_at");

    let application_id = Uuid::now_v7();
    Applications::create_application(application_id, tenant_id, &database)
        .await
        .expect("application seed should succeed");

    // subjects.created_at and users.created_at (create_user inserts the subject first, each
    // with its own in-call database sample).
    let user_id = Uuid::now_v7();
    Credentials::upsert_credential(user_id, "phc-test".to_owned(), None, &database)
        .await
        .expect("credential seed should succeed");
    let before = db_clock_now(&database)
        .await
        .expect("clock sample should succeed");
    let result = Users::create_user(
        user_id,
        application_id,
        CreateUserOpts {
            nickname: "time-authority-created-at".to_owned(),
            email: Some(format!("{user_id}@example.com")),
            phone: None,
        },
        &database,
    )
    .await
    .expect("user creation should succeed");
    let after = db_clock_now(&database)
        .await
        .expect("clock sample should succeed");
    assert_within_db_samples(
        result.subject.created_at,
        before,
        after,
        "subjects.created_at",
    );
    assert_within_db_samples(result.user.created_at, before, after, "users.created_at");
}

// NOTE: AI-generated test
#[tokio::test]
async fn trend_queries_execute_and_reflect_seeded_rows() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;
    seed_user(&database, application_id).await;

    // Creation triggers populate the summary tables; the DB-clock-anchored window must
    // include the just-seeded rows.
    let platform = trend::get_platform_trends("day", 7, &database)
        .await
        .expect("platform trend query should execute");
    assert!(
        !platform.is_empty(),
        "seeded tenant/application/user rows should appear"
    );
    for row in &platform {
        assert!(row.count > 0, "trend counts are positive");
        assert!(
            ["tenant", "application", "user", "administrator"].contains(&row.entity_type.as_str()),
            "unexpected entity_type: {}",
            row.entity_type
        );
    }
    assert!(
        platform.windows(2).all(|w| w[0].period <= w[1].period),
        "rows are ordered by period ASC"
    );

    let application = trend::get_application_trends(application_id, "day", 7, &database)
        .await
        .expect("application trend query should execute");
    assert_eq!(application.len(), 1, "one user-creation bucket expected");
    assert_eq!(application[0].count, 1);
}

// NOTE: AI-generated test
#[tokio::test]
async fn audit_summary_window_includes_events_written_at_the_db_clock() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    // An event stamped with a live database sample must fall inside the DB-clock-anchored
    // 30-day window.
    let now = db_clock_now(&database)
        .await
        .expect("clock sample should succeed");
    audits::ActiveModel {
        id: Set(Uuid::now_v7()),
        audit_type: Set(AuditType::SignJwt),
        payload: Set(json!({
            "kind": "sign_jwt",
            "data": {
                "application_id": application_id,
                "subject_id": Uuid::now_v7(),
                "jti": Uuid::now_v7(),
            }
        })),
        created_at: Set(now),
    }
    .insert(&database)
    .await
    .expect("audit event seed should succeed");

    let results = <AuditSummaryByApplication as AuditSummaryByApplicationHelper>::get_last_30days_by_application(
        application_id,
        AuditType::SignJwt,
        &database,
    )
    .await
    .expect("audit summary query should execute");

    assert_eq!(results.len(), 1, "one daily bucket expected");
    assert_eq!(results[0].event_count, 1);
    assert_eq!(
        results[0].bucket,
        now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
        "the bucket is the database-clock day at 00:00 UTC"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn audit_insert_created_at_comes_from_the_database_clock() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;

    let id = Uuid::now_v7();
    let before = db_clock_now(&database)
        .await
        .expect("clock sample should succeed");
    <Audits as oceaniam_database::helper::audits::AuditsHelper>::insert_audit_event(
        id,
        AuditType::SignJwt,
        json!({ "kind": "sign_jwt", "data": { "jti": Uuid::now_v7() } }),
        &database,
    )
    .await
    .expect("audit insert should succeed");
    let after = db_clock_now(&database)
        .await
        .expect("clock sample should succeed");

    let row = <Audits as sea_orm::EntityTrait>::find_by_id(id)
        .one(&database)
        .await
        .expect("readback should succeed")
        .expect("audit row should exist");
    assert_within_db_samples(row.created_at, before, after, "audits.created_at");
}

// NOTE: AI-generated test
#[tokio::test]
async fn managed_challenge_gate_rejects_expired_challenge_at_the_db_clock() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;
    let subject_id = seed_user(&database, application_id).await;

    let challenges = app
        .state
        .applications
        .challenges(application_id)
        .await
        .expect("managed challenges should resolve");

    // An unexpired challenge passes the gate.
    let active = challenges
        .create_challenge(subject_id, CreateChallengeOpts::new())
        .await
        .expect("challenge creation should succeed");
    challenges
        .get_challenge(active.id)
        .await
        .expect("active challenge should pass the gate");

    // A challenge whose ttl already elapsed is rejected with the same not-found error shape
    // as a missing one, evaluated against the database clock.
    let expired = challenges
        .create_challenge(
            subject_id,
            CreateChallengeOpts::new().expires_after(Duration::seconds(-1)),
        )
        .await
        .expect("challenge creation should succeed");
    let error = challenges
        .get_challenge(expired.id)
        .await
        .expect_err("expired challenge should be rejected");
    match error {
        oceaniam::error::Error::CustomMessage { code, msg, .. } => {
            assert_eq!(code, 404, "expired challenge should map to 404, got {code}");
            assert_eq!(msg, format!("challenge id={} not found", expired.id));
        }
        other => panic!("expected a 404 CustomMessage, got {other:?}"),
    }
}
