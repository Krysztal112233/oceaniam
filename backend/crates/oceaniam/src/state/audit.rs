use std::{fmt::Debug, sync::Arc};

use chrono::Utc;
use oceaniam_audit::types::AuditPayload;
use oceaniam_database::model::prelude::Audits;
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Auditing {
    writer: Arc<Box<dyn AuditWriter>>,
}

impl Auditing {
    pub fn with_database(database: DatabaseConnection) -> Self {
        Self {
            writer: Arc::new(Box::new(DatabaseAuditWriter::new(database))),
        }
    }

    pub async fn write(&self, payload: AuditPayload) {
        self.writer.write(payload).await
    }
}

#[async_trait::async_trait]
trait AuditWriter: Debug + Sync + Send {
    async fn write(&self, payload: AuditPayload);
}

#[derive(Debug, Clone)]
struct DatabaseAuditWriter {
    database: DatabaseConnection,
}

impl DatabaseAuditWriter {
    pub fn new(database: DatabaseConnection) -> DatabaseAuditWriter {
        Self { database }
    }
}

#[async_trait::async_trait]
impl AuditWriter for DatabaseAuditWriter {
    async fn write(&self, payload: AuditPayload) {
        let audit_type = payload.audit_type();
        let payload = match serde_json::to_value(payload) {
            Ok(payload) => payload,
            Err(error) => {
                error!(audit_type = %audit_type, error = %error, "failed to serialize audit payload");
                return;
            }
        };

        let _ = Audits::insert(
            oceaniam_database::model::audits::Model {
                id: Uuid::now_v7(),
                audit_type: audit_type.clone(),
                payload,
                created_at: Utc::now().into(),
            }
            .into_active_model(),
        )
        .exec_with_returning(&self.database)
        .await
        .inspect_err(|error| {
            error!(audit_type = %audit_type, error = %error, "failed to persist audit log");
        })
        .inspect(|audit| {
            info!(audit_id = %audit.id, audit_type = %audit.audit_type, "audit log persisted");
        });
    }
}
