//! Reconciles tenant key lifecycles. Repeated queue delivery must not duplicate rotations.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use linkme::distributed_slice;
use oceaniam_audit::types::{AuditPayload, RotateKeyPayload};
use oceaniam_common::consts::WORKER_ID_KEY_ROTATION;
use oceaniam_database::{
    helper::{
        SafeTransactionConnectionTrait, audits::AuditsHelper, db_clock_now,
        key_boxes::KeyBoxesHelper, tenants::TenantsHelper,
    },
    model::prelude::{Audits, KeyBoxes, Tenants},
    model::sea_orm_active_enums::KeyStatus,
};
use oceaniam_keybox::KeyBox;
use oceaniam_worker_runtime::{Worker, WorkerRuntimeError};
use sea_orm::TransactionTrait;
use tracing::{debug, info};
use uuid::Uuid;

use crate::{OceaniamWorkerRegistration, REGISTERED_WORKERS, WorkerContext, error::Error};

async fn log_key_rotation(
    tenant_id: Uuid,
    new_key_id: Uuid,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<(), Error> {
    let payload = AuditPayload::from(RotateKeyPayload {
        application_id: tenant_id,
        new_key_id,
    });
    let audit_type = payload.audit_type();
    let payload = payload
        .into_json()
        .map_err(|error| Error::internal(error.to_string()))?;
    Audits::insert_audit_event(Uuid::now_v7(), audit_type, payload, database).await?;
    Ok(())
}

#[tracing::instrument(
    level = "info",
    name = "worker.key_rotation.tenant",
    skip_all,
    fields(otel.kind = "internal", tenant.id = %tenant_id)
)]
async fn process_tenant(
    tenant_id: Uuid,
    threshold: Duration,
    context: &WorkerContext,
) -> Result<(), Error> {
    let transaction = context.database.begin().await?;
    KeyBoxes::lock_rotation(tenant_id, &transaction).await?;
    let keys = KeyBoxes::get_tenant_keys(tenant_id, &transaction).await?;
    if keys.is_empty() {
        debug!(%tenant_id, "no keys found, skipping");
        transaction.commit().await?;
        return Ok(());
    }

    let keys_map = keys.into_iter().map(|key| (key.id, key)).collect();
    let mut keybox = KeyBox::with_keys(tenant_id, keys_map, context.master_key.clone());
    let now = db_clock_now(&transaction).await?;
    let refreshed = keybox.update_keys_status(now);
    let should_rotate = keybox
        .get_keys()
        .values()
        .filter(|key| key.status == KeyStatus::Active)
        .max_by_key(|key| key.activated_at)
        .is_none_or(|key| {
            key.expires_at.signed_duration_since(now).num_days() < threshold.num_days()
        });

    if should_rotate {
        keybox.rotate(now).await?;
    }
    if refreshed || should_rotate {
        keybox.write_to(&transaction).await?;
    }
    if should_rotate
        && let Some(key) = keybox
            .get_keys()
            .values()
            .filter(|key| key.status == KeyStatus::Active)
            .max_by_key(|key| key.activated_at)
    {
        log_key_rotation(tenant_id, key.id, &transaction).await?;
        info!(%tenant_id, new_key_id = %key.id, "key rotation completed");
    }
    transaction.commit().await?;
    Ok(())
}

struct KeyRotationWorker;

#[async_trait]
impl Worker<WorkerContext> for KeyRotationWorker {
    type Error = Error;

    fn name(&self) -> &'static str {
        WORKER_ID_KEY_ROTATION
    }

    async fn run(&self, context: &WorkerContext) -> Result<(), Error> {
        let tenants = Tenants::list_all_tenants(&context.database).await?;
        // Keep the existing policy; changing rotation cadence/lifetimes is a separate concern.
        let threshold = Duration::days(7);
        for tenant in &tenants {
            process_tenant(tenant.id, threshold, context).await?;
        }
        Ok(())
    }
}

#[distributed_slice(REGISTERED_WORKERS)]
static KEY_ROTATION_WORKER: fn() -> OceaniamWorkerRegistration = || OceaniamWorkerRegistration {
    worker: Arc::new(KeyRotationWorker),
    cron: Some("0 0 */6 * * *"),
};
