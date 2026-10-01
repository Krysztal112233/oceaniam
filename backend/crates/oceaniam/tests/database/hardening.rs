//! Behavior coverage for the hardened helper write paths and the new cleanup-slice helpers:
//! atomic credential upsert, conditional challenge consume, delete/update row-existence
//! reporting, subject expiration reporting, administrator/tenant probes, and the pgmq wrapper.

use chrono::{Duration, Utc};
use oceaniam_database::error::Error as DbError;
use oceaniam_database::helper::administrator_tenants::AdministratorTenantsHelper;
use oceaniam_database::helper::administrators::{AdministratorsHelper, UpdateAdministratorModel};
use oceaniam_database::helper::application_roles::ApplicationRolesHelper;
use oceaniam_database::helper::applications::ApplicationHelper;
use oceaniam_database::helper::challenges::{ChallengesHelper, CreateChallengeOpts};
use oceaniam_database::helper::credentials::CredentialsHelper;
use oceaniam_database::helper::pgmq;
use oceaniam_database::helper::subjects::SubjectsHelper;
use oceaniam_database::helper::tenants::TenantsHelper;
use oceaniam_database::helper::users::{CreateUserOpts, PatchUserOpts, UserHelper};
use oceaniam_database::model::sea_orm_active_enums::ChallengeStatusType;
use oceaniam_database::model::{
    self,
    prelude::{
        AdministratorTenants, Administrators, ApplicationRoles, Applications, Challenges,
        Credentials, Subjects, Tenants, Users,
    },
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseBackend, FromQueryResult, Statement};
use sea_orm::{ConnectionTrait, DatabaseConnection};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::support::spawn_app_with_isolated_schema;

/// Asserts that a helper error carries the not-found (404) code.
fn assert_not_found(error: &DbError, context: &str) {
    match error {
        DbError::CustomMessage { code, .. } => {
            assert_eq!(*code, 404, "{context}: expected 404, got {code}")
        }
        other => panic!("{context}: expected a 404 CustomMessage, got {other:?}"),
    }
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
            nickname: "hardening-test".to_owned(),
            email: Some(format!("{user_id}@example.com")),
            phone: None,
        },
        database,
    )
    .await
    .expect("user seed should succeed");
    user_id
}

/// Seeds a credential + administrator pair (administrators.id references credentials.id).
async fn seed_administrator(database: &DatabaseConnection, name: &str) -> Uuid {
    let administrator_id = Uuid::now_v7();
    Credentials::upsert_credential(administrator_id, "phc-admin".to_owned(), None, database)
        .await
        .expect("credential seed should succeed");
    Administrators::create_administrator(administrator_id, name, database)
        .await
        .expect("administrator seed should succeed");
    administrator_id
}

// NOTE: AI-generated test
#[tokio::test]
async fn upsert_credential_inserts_then_updates_on_conflict() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let id = Uuid::now_v7();

    let inserted = Credentials::upsert_credential(id, "phc-one".to_owned(), None, &database)
        .await
        .expect("initial upsert should insert");
    assert_eq!(inserted.phc, "phc-one");
    assert_eq!(inserted.totp, None);

    let updated = Credentials::upsert_credential(
        id,
        "phc-two".to_owned(),
        Some("totp-secret".to_owned()),
        &database,
    )
    .await
    .expect("conflicting upsert should take the ON CONFLICT DO UPDATE path");
    assert_eq!(updated.phc, "phc-two");
    assert_eq!(updated.totp.as_deref(), Some("totp-secret"));

    let stored = Credentials::get_credential_by_id(id, &database)
        .await
        .expect("get should succeed")
        .expect("credential should exist");
    assert_eq!(stored.phc, "phc-two");
}

// NOTE: AI-generated test
#[tokio::test]
async fn upsert_credential_concurrent_same_id_both_succeed() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let id = Uuid::now_v7();

    let (first, second) = tokio::join!(
        Credentials::upsert_credential(id, "phc-a".to_owned(), None, &database),
        Credentials::upsert_credential(
            id,
            "phc-b".to_owned(),
            Some("totp-b".to_owned()),
            &database
        ),
    );
    first.expect("first concurrent upsert should succeed");
    second.expect("second concurrent upsert should succeed");

    let stored = Credentials::get_credential_by_id(id, &database)
        .await
        .expect("get should succeed")
        .expect("credential should exist");
    assert!(
        stored.phc == "phc-a" || stored.phc == "phc-b",
        "final row must be one of the two concurrent upserts, got {}",
        stored.phc
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn delete_credential_by_id_reports_missing_row() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;

    let missing = Credentials::delete_credential_by_id(Uuid::now_v7(), &database)
        .await
        .expect_err("deleting a missing credential should fail");
    assert_not_found(&missing, "delete_credential_by_id");

    let id = Uuid::now_v7();
    Credentials::upsert_credential(id, "phc".to_owned(), None, &database)
        .await
        .expect("upsert should succeed");
    Credentials::delete_credential_by_id(id, &database)
        .await
        .expect("deleting an existing credential should succeed");

    let gone = Credentials::delete_credential_by_id(id, &database)
        .await
        .expect_err("deleting the same credential twice should fail");
    assert_not_found(&gone, "delete_credential_by_id second delete");
}

// NOTE: AI-generated test
#[tokio::test]
async fn consume_challenge_rejects_already_consumed_challenge() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;
    let subject_id = seed_user(&database, application_id).await;

    let challenge = Challenges::create_challenge(
        application_id,
        subject_id,
        CreateChallengeOpts::new(),
        &database,
    )
    .await
    .expect("challenge seed should succeed");

    let consumed = Challenges::consume_challenge(challenge.id, &database)
        .await
        .expect("first consume should succeed");
    assert_eq!(consumed.status, ChallengeStatusType::Consumed);
    assert!(consumed.consumed_at.is_some());

    let second = Challenges::consume_challenge(challenge.id, &database)
        .await
        .expect_err("consuming an already-consumed challenge should fail");
    assert_not_found(&second, "consume_challenge double consume");
}

// NOTE: AI-generated test
#[tokio::test]
async fn consume_challenge_rejects_expired_challenge() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;
    let subject_id = seed_user(&database, application_id).await;

    let challenge = Challenges::create_challenge(
        application_id,
        subject_id,
        CreateChallengeOpts::new().expires_after(Duration::seconds(-5)),
        &database,
    )
    .await
    .expect("challenge seed should succeed");

    let error = Challenges::consume_challenge(challenge.id, &database)
        .await
        .expect_err("consuming an expired challenge should fail");
    assert_not_found(&error, "consume_challenge expired");

    let stored = Challenges::get_challenge(challenge.id, &database)
        .await
        .expect("expired challenge row should still exist");
    assert_eq!(
        stored.status,
        ChallengeStatusType::Pending,
        "rejected consume must not alter the challenge"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn delete_helpers_report_missing_rows() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (tenant_id, application_id) = app.seed_tenant_and_application().await;

    let missing_tenant = Tenants::delete_tenant(Uuid::now_v7(), &database)
        .await
        .expect_err("deleting a missing tenant should fail");
    assert_not_found(&missing_tenant, "delete_tenant");

    let missing_application = Applications::delete_application(Uuid::now_v7(), &database)
        .await
        .expect_err("deleting a missing application should fail");
    assert_not_found(&missing_application, "delete_application");

    let missing_role = ApplicationRoles::delete_role(Uuid::now_v7(), &database)
        .await
        .expect_err("deleting a missing role should fail");
    assert_not_found(&missing_role, "delete_role");

    let missing_user = Users::delete_user(Uuid::now_v7(), &database)
        .await
        .expect_err("deleting a missing user should fail");
    assert_not_found(&missing_user, "delete_user");

    let role_id = Uuid::now_v7();
    ApplicationRoles::create_role(role_id, application_id, "role".to_owned(), false, &database)
        .await
        .expect("role seed should succeed");
    ApplicationRoles::delete_role(role_id, &database)
        .await
        .expect("deleting an existing role should succeed");

    let user_id = seed_user(&database, application_id).await;
    Users::delete_user(user_id, &database)
        .await
        .expect("deleting an existing user should succeed");

    Applications::delete_application(application_id, &database)
        .await
        .expect("deleting an existing application should succeed");
    Tenants::delete_tenant(tenant_id, &database)
        .await
        .expect("deleting an existing tenant should succeed");
}

// NOTE: AI-generated test
#[tokio::test]
async fn update_helpers_report_missing_rows() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;

    let missing_tenant = Tenants::update_comment(Uuid::now_v7(), Some("c".to_owned()), &database)
        .await
        .expect_err("updating a missing tenant should fail");
    assert_not_found(&missing_tenant, "tenants::update_comment");

    let missing_administrator = Administrators::update_model(
        Uuid::now_v7(),
        UpdateAdministratorModel {
            name: Some("n".to_owned()),
        },
        &database,
    )
    .await
    .expect_err("updating a missing administrator should fail");
    assert_not_found(&missing_administrator, "administrators::update_model");

    let missing_role =
        ApplicationRoles::update_role_name(Uuid::now_v7(), "n".to_owned(), &database)
            .await
            .expect_err("updating a missing role should fail");
    assert_not_found(&missing_role, "application_roles::update_role_name");

    let missing_configuration =
        Applications::replace_configuration(Uuid::now_v7(), Default::default(), &database)
            .await
            .expect_err("replacing configuration of a missing application should fail");
    assert_not_found(
        &missing_configuration,
        "applications::replace_configuration",
    );

    let missing_application_comment =
        Applications::update_comment(Uuid::now_v7(), Some("c".to_owned()), &database)
            .await
            .expect_err("updating a missing application should fail");
    assert_not_found(&missing_application_comment, "applications::update_comment");

    let missing_user = Users::patch_user(
        Uuid::now_v7(),
        Uuid::now_v7(),
        PatchUserOpts {
            nickname: Some("n".to_owned()),
            email: None,
            phone: None,
        },
        &database,
    )
    .await
    .expect_err("patching a missing user should fail");
    assert_not_found(&missing_user, "users::patch_user");
}

// NOTE: AI-generated test
#[tokio::test]
async fn update_helpers_apply_changes_to_existing_rows() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (tenant_id, application_id) = app.seed_tenant_and_application().await;

    let tenant = Tenants::update_comment(tenant_id, Some("updated".to_owned()), &database)
        .await
        .expect("updating an existing tenant should succeed");
    assert_eq!(tenant.comment.as_deref(), Some("updated"));

    let application_comment =
        Applications::update_comment(application_id, Some("app-comment".to_owned()), &database)
            .await
            .expect("updating an existing application should succeed");
    assert_eq!(application_comment.comment.as_deref(), Some("app-comment"));

    let role_id = Uuid::now_v7();
    ApplicationRoles::create_role(
        role_id,
        application_id,
        "before".to_owned(),
        false,
        &database,
    )
    .await
    .expect("role seed should succeed");
    let role = ApplicationRoles::update_role_name(role_id, "after".to_owned(), &database)
        .await
        .expect("renaming an existing role should succeed");
    assert_eq!(role.name, "after");

    let administrator_id = seed_administrator(&database, "admin-before").await;
    let administrator = Administrators::update_model(
        administrator_id,
        UpdateAdministratorModel {
            name: Some("admin-after".to_owned()),
        },
        &database,
    )
    .await
    .expect("updating an existing administrator should succeed");
    assert_eq!(administrator.name, "admin-after");

    // A field-less patch is a no-op read of the current row.
    let unchanged = Administrators::update_model(
        administrator_id,
        UpdateAdministratorModel { name: None },
        &database,
    )
    .await
    .expect("a field-less administrator patch should succeed as a no-op");
    assert_eq!(unchanged.name, "admin-after");

    let user_id = seed_user(&database, application_id).await;
    let user = Users::patch_user(
        application_id,
        user_id,
        PatchUserOpts {
            nickname: Some("patched".to_owned()),
            email: None,
            phone: None,
        },
        &database,
    )
    .await
    .expect("patching an existing user should succeed");
    assert_eq!(user.nickname, "patched");
}

// NOTE: AI-generated test
#[tokio::test]
async fn set_subject_expiration_reports_missing_subject() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    let expires_at = (Utc::now() + Duration::hours(1)).fixed_offset();
    let missing = Subjects::set_subject_expiration(Uuid::now_v7(), expires_at, &database)
        .await
        .expect_err("stamping a missing subject should fail");
    assert_not_found(&missing, "set_subject_expiration");

    let subject_id = seed_user(&database, application_id).await;
    let subject = Subjects::set_subject_expiration(subject_id, expires_at, &database)
        .await
        .expect("stamping an existing subject should succeed");
    // timestamptz stores microsecond precision; compare at that resolution.
    assert_eq!(
        subject.expires_at.map(|it| it.timestamp_micros()),
        Some(expires_at.timestamp_micros())
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn administrator_helpers_cover_lookup_and_assignment_probe() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (tenant_id, _application_id) = app.seed_tenant_and_application().await;

    let missing = Administrators::get_administrator(Uuid::now_v7(), &database)
        .await
        .expect_err("looking up a missing administrator should fail");
    assert_not_found(&missing, "get_administrator");

    let administrator_id = seed_administrator(&database, "scoped-admin").await;
    let administrator = Administrators::get_administrator(administrator_id, &database)
        .await
        .expect("looking up an existing administrator should succeed");
    assert_eq!(administrator.name, "scoped-admin");

    assert!(
        !AdministratorTenants::assignment_exists(administrator_id, tenant_id, &database)
            .await
            .expect("assignment probe should succeed"),
        "unassigned administrator should not have tenant scope"
    );

    model::administrator_tenants::ActiveModel {
        id: Set(Uuid::now_v7()),
        administrator_id: Set(administrator_id),
        tenant_id: Set(tenant_id),
    }
    .insert(&database)
    .await
    .expect("assignment seed should succeed");

    assert!(
        AdministratorTenants::assignment_exists(administrator_id, tenant_id, &database)
            .await
            .expect("assignment probe should succeed"),
        "assigned administrator should have tenant scope"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn tenant_exists_probes_regular_and_system_tenants() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (tenant_id, _application_id) = app.seed_tenant_and_application().await;

    assert!(
        Tenants::tenant_exists(tenant_id, &database)
            .await
            .expect("probe should succeed"),
        "created tenant should exist"
    );
    assert!(
        !Tenants::tenant_exists(Uuid::now_v7(), &database)
            .await
            .expect("probe should succeed"),
        "random id should not exist"
    );
    assert!(
        Tenants::tenant_exists(oceaniam_common::consts::SYSTEM_TENANT_UUID, &database)
            .await
            .expect("probe should succeed"),
        "the probe deliberately includes the system tenant (keybox auto-bootstrap)"
    );
}

#[derive(Debug, FromQueryResult)]
struct TestQueueRow {
    msg_id: i64,
    #[allow(dead_code)]
    read_ct: i32,
    message: Value,
}

// NOTE: AI-generated test
#[tokio::test]
async fn pgmq_helpers_send_read_delete_and_archive() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;

    // Unique per-test queue: the `pgmq` schema is database-global (shared across isolated test
    // schemas), so a dedicated queue keeps this test isolated; it is dropped at the end. pgmq
    // caps queue names at 47 characters.
    let queue = format!("ch_{}", Uuid::now_v7().simple());
    database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT pgmq.create($1)",
            [queue.clone().into()],
        ))
        .await
        .expect("queue creation should succeed");

    let payload = json!({ "purpose": "cleanup-hardening-test" });

    let msg_id = pgmq::send(&queue, payload.clone(), 0, &database)
        .await
        .expect("send should return a message id");
    let rows = pgmq::read::<TestQueueRow>(&queue, 60, 10, &database)
        .await
        .expect("read should succeed");
    assert_eq!(rows.len(), 1, "one message should be visible");
    assert_eq!(rows[0].msg_id, msg_id);
    assert_eq!(rows[0].message, payload);

    pgmq::delete(&queue, msg_id, &database)
        .await
        .expect("delete should succeed");
    let rows = pgmq::read::<TestQueueRow>(&queue, 60, 10, &database)
        .await
        .expect("read after delete should succeed");
    assert!(rows.is_empty(), "deleted message must not reappear");

    let msg_id = pgmq::send(&queue, payload.clone(), 0, &database)
        .await
        .expect("second send should succeed");
    let rows = pgmq::read::<TestQueueRow>(&queue, 60, 10, &database)
        .await
        .expect("second read should succeed");
    assert_eq!(rows.len(), 1, "second message should be visible");
    pgmq::archive(&queue, msg_id, &database)
        .await
        .expect("archive should succeed");
    let rows = pgmq::read::<TestQueueRow>(&queue, 60, 10, &database)
        .await
        .expect("read after archive should succeed");
    assert!(rows.is_empty(), "archived message must not reappear");

    // The archive table keeps the dead-lettered message.
    let archived = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            format!("SELECT COUNT(*) AS archived FROM pgmq.a_{queue}"),
            [],
        ))
        .await
        .expect("archive count query should succeed")
        .expect("archive count row should exist")
        .try_get::<i64>("", "archived")
        .expect("archive count should decode");
    assert_eq!(archived, 1, "archived message should be dead-lettered");

    database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT pgmq.drop_queue($1)",
            [queue.clone().into()],
        ))
        .await
        .expect("queue cleanup should succeed");
}
