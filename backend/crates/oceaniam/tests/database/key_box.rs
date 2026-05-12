use chrono::{DateTime, Duration, FixedOffset, Utc};
use oceaniam_database::model::sea_orm_active_enums::{KeyAlg, KeyStatus};
use oceaniam_database::model::{self, key_boxes};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde_json::json;
use uuid::Uuid;

use crate::support::spawn_app_with_isolated_schema;

async fn insert_key_box(
    db: &sea_orm::DatabaseConnection,
    id: Uuid,
    application_id: Uuid,
    now: DateTime<FixedOffset>,
    activated_at: DateTime<FixedOffset>,
    retired_at: DateTime<FixedOffset>,
    expires_at: DateTime<FixedOffset>,
) -> Result<model::key_boxes::Model, sea_orm::DbErr> {
    key_boxes::ActiveModel {
        id: Set(id),
        key_alg: Set(KeyAlg::Rs256),
        status: Set(KeyStatus::Active),
        created_at: Set(now),
        activated_at: Set(activated_at),
        retired_at: Set(retired_at),
        revoked_at: Set(None),
        expires_at: Set(expires_at),
        secret: Set(json!({"k": "v"})),
        application_id: Set(application_id),
    }
    .insert(db)
    .await
}

// NOTE: AI-generated test
#[tokio::test]
async fn valid_key_boxes_timestamps_passes_check_constraint() {
    let app = spawn_app_with_isolated_schema().await;
    let db = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    let now: DateTime<FixedOffset> = Utc::now().into();
    let key_id = Uuid::now_v7();

    insert_key_box(
        &db,
        key_id,
        application_id,
        now,
        now,
        now + Duration::hours(1),
        now + Duration::hours(2),
    )
    .await
    .expect("INSERT with activated_at < retired_at < expires_at should succeed");
}

// NOTE: AI-generated test
#[tokio::test]
async fn activated_at_after_retired_at_violates_check_constraint() {
    let app = spawn_app_with_isolated_schema().await;
    let db = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    let now: DateTime<FixedOffset> = Utc::now().into();
    let key_id = Uuid::now_v7();

    let err = insert_key_box(
        &db,
        key_id,
        application_id,
        now,
        now,
        now - Duration::hours(1),
        now + Duration::hours(2),
    )
    .await
    .expect_err("INSERT with activated_at > retired_at should fail");

    assert!(
        err.to_string().contains("ck_key_boxes_temporal_order"),
        "error should mention the constraint name, got: {err}"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn retired_at_after_expires_at_violates_check_constraint() {
    let app = spawn_app_with_isolated_schema().await;
    let db = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    let now: DateTime<FixedOffset> = Utc::now().into();
    let key_id = Uuid::now_v7();

    let err = insert_key_box(
        &db,
        key_id,
        application_id,
        now,
        now - Duration::hours(2),
        now + Duration::hours(2),
        now + Duration::hours(1),
    )
    .await
    .expect_err("INSERT with retired_at > expires_at should fail");

    assert!(
        err.to_string().contains("ck_key_boxes_temporal_order"),
        "error should mention the constraint name, got: {err}"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn update_that_violates_constraint_is_rejected() {
    let app = spawn_app_with_isolated_schema().await;
    let db = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    let now: DateTime<FixedOffset> = Utc::now().into();
    let key_id = Uuid::now_v7();

    // Insert a valid row first
    insert_key_box(
        &db,
        key_id,
        application_id,
        now,
        now,
        now + Duration::days(1),
        now + Duration::days(2),
    )
    .await
    .expect("initial INSERT should succeed");

    // Update retired_at to exceed expires_at — should trigger the constraint
    let result = key_boxes::ActiveModel {
        id: Set(key_id),
        retired_at: Set(now + Duration::days(3)),
        ..Default::default()
    }
    .update(&db)
    .await;

    let err = result.expect_err("UPDATE that makes retired_at > expires_at should fail");
    assert!(
        err.to_string().contains("ck_key_boxes_temporal_order"),
        "error should mention the constraint name, got: {err}"
    );
}
