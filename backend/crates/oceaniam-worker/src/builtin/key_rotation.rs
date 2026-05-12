use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use linkme::distributed_slice;
use oceaniam_audit::types::{AuditPayload, RotateKeyPayload};
use oceaniam_common::error::Error;
use oceaniam_database::{
    helper::key_boxes::KeyBoxesHelper,
    model::{
        self,
        prelude::{Applications, KeyBoxes},
        sea_orm_active_enums::KeyStatus,
    },
};
use oceaniam_keybox::KeyBox;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::runtime::{REGISTERED_WORKERS, Worker, WorkerContext, WorkerRef};

async fn log_key_rotation(
    application_id: Uuid,
    new_key_id: Uuid,

    database: &sea_orm::DatabaseConnection,
) {
    let payload = AuditPayload::from(RotateKeyPayload {
        application_id,
        new_key_id,
    });

    let model = model::audits::ActiveModel {
        id: ActiveValue::Set(Uuid::now_v7()),
        audit_type: ActiveValue::Set(payload.audit_type()),
        payload: ActiveValue::Set(payload.into_json().unwrap_or_default()),
        created_at: ActiveValue::Set(Utc::now().into()),
    };

    if let Err(e) = model.insert(database).await {
        error!(%application_id, %new_key_id, error = %e, "failed to write rotation audit event");
    }
}

struct KeyRotationWorker;

#[async_trait]
impl Worker for KeyRotationWorker {
    fn name(&self) -> &'static str {
        "key_rotation"
    }

    fn cron(&self) -> &'static str {
        "0 0 */6 * * *"
    }

    async fn run(&self, context: &WorkerContext) -> Result<(), Error> {
        let apps = Applications::find().all(&context.database).await?;

        // TODO: make this configurable via ApplicationConfiguration fields. Each application should
        // be able to control whether auto-rotation is enabled, the threshold duration, and the
        // lifetime of rotated keys.
        let threshold = Duration::days(7);

        for app in &apps {
            let keys = KeyBoxes::get_application_keys(app.id, &context.database).await?;

            if keys.is_empty() {
                debug!(application_id = %app.id, "no keys found, skipping");
                continue;
            }

            let keys_map: im::HashMap<_, _> = keys.into_iter().map(|k| (k.id, k)).collect();
            let keybox = KeyBox::with_keys(app.id, keys_map);
            let should_rotate = keybox
                .get_keys()
                .values()
                .filter(|k| k.status == KeyStatus::Active)
                .max_by_key(|k| k.activated_at)
                .map_or(true, |key| {
                    key.expires_at.is_none_or(|exp| {
                        exp.signed_duration_since(Utc::now()).num_days() < threshold.num_days()
                    })
                });

            if should_rotate {
                let mut keybox = keybox;
                match keybox.rotate_key() {
                    Ok(new_key) => {
                        if let Err(e) = keybox.write_to(&context.database).await {
                            error!(%app.id, error = %e, "failed to persist rotated key");
                        } else {
                            info!(%app.id, new_key_id = %new_key.id, "key rotation completed");
                            log_key_rotation(app.id, new_key.id, &context.database).await;
                        }
                    }
                    Err(e) => {
                        error!(%app.id, error = %e, "failed to rotate key");
                    }
                }
            }
        }

        Ok(())
    }
}

#[distributed_slice(REGISTERED_WORKERS)]
static KEY_ROTATION_WORKER: fn() -> WorkerRef = || Arc::new(KeyRotationWorker);
