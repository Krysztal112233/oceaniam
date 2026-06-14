use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use linkme::distributed_slice;
use oceaniam_audit::types::{AuditPayload, RotateKeyPayload};
use oceaniam_database::{
    helper::{audits::AuditsHelper, key_boxes::KeyBoxesHelper, tenants::TenantsHelper},
    model::prelude::{Audits, KeyBoxes, Tenants},
    model::sea_orm_active_enums::KeyStatus,
};
use oceaniam_keybox::KeyBox;
use oceaniam_worker_runtime::Worker;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{REGISTERED_WORKERS, WorkerContext, error::Error};

async fn log_key_rotation(
    tenant_id: Uuid,
    new_key_id: Uuid,
    database: &sea_orm::DatabaseConnection,
) {
    let payload = AuditPayload::from(RotateKeyPayload {
        application_id: tenant_id,
        new_key_id,
    });

    if let Err(e) = Audits::insert_audit_event(
        Uuid::now_v7(),
        payload.audit_type(),
        payload.into_json().unwrap_or_default(),
        database,
    )
    .await
    {
        error!(%tenant_id, %new_key_id, error = %e, "failed to write rotation audit event");
    }
}

struct KeyRotationWorker;

#[async_trait]
impl Worker<WorkerContext> for KeyRotationWorker {
    type Error = Error;

    fn name(&self) -> &'static str {
        "key_rotation"
    }

    fn cron(&self) -> &'static str {
        "0 0 */6 * * *"
    }

    async fn run(&self, context: &WorkerContext) -> Result<(), Error> {
        let tenants = Tenants::list_all_tenants(&context.database).await?;

        // TODO: make this configurable via ApplicationConfiguration fields. Each application should
        // be able to control whether auto-rotation is enabled, the threshold duration, and the
        // lifetime of rotated keys.
        let threshold = Duration::days(7);

        for tenant in &tenants {
            let keys = KeyBoxes::get_tenant_keys(tenant.id, &context.database).await?;

            if keys.is_empty() {
                debug!(tenant_id = %tenant.id, "no keys found, skipping");
                continue;
            }

            let keys_map = keys.into_iter().map(|k| (k.id, k)).collect();
            let mut keybox = KeyBox::with_keys(tenant.id, keys_map, (*context.master_key).clone());

            if keybox.update_keys_status() {
                debug!(tenant_id = %tenant.id, "key statuses refreshed");
                if let Err(e) = keybox.write_to(&context.database).await {
                    error!(%tenant.id, error = %e, "failed to persist refreshed key statuses");
                }
            }

            let should_rotate = keybox
                .get_keys()
                .values()
                .filter(|k| k.status == KeyStatus::Active)
                .max_by_key(|k| k.activated_at)
                .is_none_or(|key| {
                    key.expires_at.signed_duration_since(Utc::now()).num_days()
                        < threshold.num_days()
                });

            if should_rotate {
                match keybox.rotate() {
                    Ok(()) => {
                        if let Err(e) = keybox.write_to(&context.database).await {
                            error!(%tenant.id, error = %e, "failed to persist rotated key");
                        } else {
                            let latest_active = keybox
                                .get_latest_raw_key(KeyStatus::Active)
                                .and_then(|raw| keybox.get_raw_key_unchecked(&raw.key_id));
                            if let Some(ref new_key) = latest_active {
                                info!(%tenant.id, new_key_id = %new_key.id, "key rotation completed");
                                log_key_rotation(tenant.id, new_key.id, &context.database).await;
                            }
                        }
                    }
                    Err(e) => {
                        error!(%tenant.id, error = %e, "failed to rotate key");
                    }
                }
            }
        }

        Ok(())
    }
}

#[distributed_slice(REGISTERED_WORKERS)]
static KEY_ROTATION_WORKER: fn() -> crate::OceaniamWorkerRef = || Arc::new(KeyRotationWorker);
