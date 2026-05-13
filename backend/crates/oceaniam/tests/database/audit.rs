use chrono::{DateTime, Duration, FixedOffset, Utc};
use oceaniam_database::{
    helper::{
        applications::ApplicationHelper,
        audit_summary_by_application::AuditSummaryByApplicationHelper,
    },
    model::{
        audits, prelude::Applications, prelude::AuditSummaryByApplication,
        sea_orm_active_enums::AuditType,
    },
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde_json::json;
use uuid::Uuid;

use crate::support::spawn_app_with_isolated_schema;

async fn insert_audit_event(
    db: &sea_orm::DatabaseConnection,
    application_id: Uuid,
    audit_type: AuditType,
    created_at: DateTime<FixedOffset>,
) {
    let payload = json!({
        "kind": "sign_jwt",
        "data": {
            "application_id": application_id,
            "subject_id": Uuid::now_v7(),
            "jti": Uuid::now_v7(),
        }
    });

    audits::ActiveModel {
        id: Set(Uuid::now_v7()),
        audit_type: Set(audit_type),
        payload: Set(payload),
        created_at: Set(created_at),
    }
    .insert(db)
    .await
    .expect("failed to insert test audit event");
}

fn midnight_utc(dt: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
    dt.date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .into()
}

// NOTE: AI-generated test
#[tokio::test]
async fn aggregates_same_day_minutes_into_one_row() {
    let app = spawn_app_with_isolated_schema().await;
    let db = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    let now: DateTime<FixedOffset> = Utc::now().into();

    insert_audit_event(&db, application_id, AuditType::SignJwt, now).await;
    insert_audit_event(
        &db,
        application_id,
        AuditType::SignJwt,
        now + Duration::minutes(5),
    )
    .await;
    insert_audit_event(
        &db,
        application_id,
        AuditType::SignJwt,
        now + Duration::minutes(30),
    )
    .await;

    let results = <AuditSummaryByApplication as AuditSummaryByApplicationHelper>::get_last_30days_by_application(
        application_id,
        AuditType::SignJwt,
        &db,
    )
    .await
    .unwrap();

    assert_eq!(results.len(), 1, "should aggregate into a single daily row");
    assert_eq!(results[0].event_count, 3);
    assert_eq!(results[0].bucket, midnight_utc(now));
}

// NOTE: AI-generated test
#[tokio::test]
async fn returns_multiple_days() {
    let app = spawn_app_with_isolated_schema().await;
    let db = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    let now: DateTime<FixedOffset> = Utc::now().into();

    // Day -2: 1 event
    insert_audit_event(
        &db,
        application_id,
        AuditType::SignJwt,
        now - Duration::days(2),
    )
    .await;
    // Day -1: 3 events
    insert_audit_event(
        &db,
        application_id,
        AuditType::SignJwt,
        now - Duration::days(1),
    )
    .await;
    insert_audit_event(
        &db,
        application_id,
        AuditType::SignJwt,
        now - Duration::days(1) + Duration::minutes(10),
    )
    .await;
    insert_audit_event(
        &db,
        application_id,
        AuditType::SignJwt,
        now - Duration::days(1) + Duration::hours(1),
    )
    .await;
    // Day 0 (today): 2 events
    insert_audit_event(&db, application_id, AuditType::SignJwt, now).await;
    insert_audit_event(
        &db,
        application_id,
        AuditType::SignJwt,
        now + Duration::minutes(15),
    )
    .await;

    let results = <AuditSummaryByApplication as AuditSummaryByApplicationHelper>::get_last_30days_by_application(
        application_id,
        AuditType::SignJwt,
        &db,
    )
    .await
    .unwrap();

    assert_eq!(results.len(), 3, "should return one row per day");

    // Ordered by day ascending: day -2, day -1, today
    assert_eq!(results[0].event_count, 1);
    assert_eq!(results[0].bucket, midnight_utc(now - Duration::days(2)));

    assert_eq!(results[1].event_count, 3);
    assert_eq!(results[1].bucket, midnight_utc(now - Duration::days(1)));

    assert_eq!(results[2].event_count, 2);
    assert_eq!(results[2].bucket, midnight_utc(now));
}

// NOTE: AI-generated test
#[tokio::test]
async fn filters_by_application_id() {
    let app = spawn_app_with_isolated_schema().await;
    let db = app.database().await;
    let (tenant_id, app_a) = app.seed_tenant_and_application().await;

    // Seed a second application under the same tenant
    let app_b = Uuid::now_v7();
    <Applications as ApplicationHelper>::create_application(app_b, tenant_id, &db)
        .await
        .unwrap();

    let now: DateTime<FixedOffset> = Utc::now().into();

    insert_audit_event(&db, app_a, AuditType::SignJwt, now).await;
    insert_audit_event(&db, app_a, AuditType::SignJwt, now + Duration::minutes(1)).await;
    insert_audit_event(&db, app_b, AuditType::SignJwt, now).await;

    let results = <AuditSummaryByApplication as AuditSummaryByApplicationHelper>::get_last_30days_by_application(
        app_a,
        AuditType::SignJwt,
        &db,
    )
    .await
    .unwrap();

    assert_eq!(results.len(), 1, "should only return rows for app_a");
    assert_eq!(results[0].event_count, 2);
}

// NOTE: AI-generated test
#[tokio::test]
async fn filters_by_audit_type() {
    let app = spawn_app_with_isolated_schema().await;
    let db = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    let now: DateTime<FixedOffset> = Utc::now().into();

    insert_audit_event(&db, application_id, AuditType::SignJwt, now).await;
    insert_audit_event(
        &db,
        application_id,
        AuditType::SignJwt,
        now + Duration::minutes(1),
    )
    .await;
    insert_audit_event(
        &db,
        application_id,
        AuditType::CreateTenants,
        now + Duration::minutes(2),
    )
    .await;

    let results = <AuditSummaryByApplication as AuditSummaryByApplicationHelper>::get_last_30days_by_application(
        application_id,
        AuditType::SignJwt,
        &db,
    )
    .await
    .unwrap();

    assert_eq!(results.len(), 1, "should only return rows for SignJwt");
    assert_eq!(results[0].event_count, 2);
}

// NOTE: AI-generated test
#[tokio::test]
async fn respects_30_day_window() {
    let app = spawn_app_with_isolated_schema().await;
    let db = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    let now: DateTime<FixedOffset> = Utc::now().into();

    // Event within the 30-day window: t-29 days
    insert_audit_event(
        &db,
        application_id,
        AuditType::SignJwt,
        now - Duration::days(29),
    )
    .await;
    // Event outside the 30-day window: t-31 days
    insert_audit_event(
        &db,
        application_id,
        AuditType::SignJwt,
        now - Duration::days(31),
    )
    .await;

    let results = <AuditSummaryByApplication as AuditSummaryByApplicationHelper>::get_last_30days_by_application(
        application_id,
        AuditType::SignJwt,
        &db,
    )
    .await
    .unwrap();

    assert_eq!(
        results.len(),
        1,
        "should exclude events outside the 30-day window"
    );
    assert_eq!(results[0].event_count, 1);
}

// NOTE: AI-generated test
#[tokio::test]
async fn returns_empty_for_no_events() {
    let app = spawn_app_with_isolated_schema().await;
    let db = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;

    let results = <AuditSummaryByApplication as AuditSummaryByApplicationHelper>::get_last_30days_by_application(
        application_id,
        AuditType::SignJwt,
        &db,
    )
    .await
    .unwrap();

    assert!(
        results.is_empty(),
        "should return empty vec when no events exist"
    );
}
