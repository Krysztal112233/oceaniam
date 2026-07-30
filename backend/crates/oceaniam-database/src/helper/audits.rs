use chrono::Utc;
use sea_orm::{EntityTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Audits, sea_orm_active_enums::AuditType},
};

pub fn audit_model_to_active_model(model: model::audits::Model) -> model::audits::ActiveModel {
    model.into_active_model()
}

#[async_trait::async_trait]
pub trait AuditsHelper {
    #[tracing::instrument(
        level = "info",
        name = "db.audits.insert_many_audits",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn insert_many_audits(
        models: Vec<model::audits::ActiveModel>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        Audits::insert_many(models).exec(database).await?;
        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "db.audits.insert_audit_event",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn insert_audit_event(
        id: Uuid,
        audit_type: AuditType,
        payload: serde_json::Value,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        let model = model::audits::Model {
            id,
            audit_type,
            payload,
            created_at: Utc::now().into(),
        }
        .into_active_model();

        Audits::insert_many(vec![model]).exec(database).await?;
        Ok(())
    }
}

impl AuditsHelper for Audits {}
