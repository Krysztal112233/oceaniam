use std::{sync::Arc, time::Duration};

use chrono::Utc;
use crossbeam_queue::SegQueue;
use oceaniam_audit::types::AuditPayload;
use oceaniam_database::{
    helper::audits::{AuditsHelper, audit_model_to_active_model},
    model::{self, prelude::Audits},
};
use sea_orm::DatabaseConnection;
use tokio::time::sleep;
use uuid::Uuid;

use super::AuditWriter;

type AuditActiveModel = model::audits::ActiveModel;

#[derive(Debug, Clone)]
pub(super) struct DatabaseAuditWriter {
    queue: Arc<SegQueue<AuditActiveModel>>,
    database: DatabaseConnection,
}

impl DatabaseAuditWriter {
    pub fn new(database: DatabaseConnection) -> DatabaseAuditWriter {
        let queue = Arc::new(SegQueue::new());

        let it = Self { queue, database };

        let worker = it.clone();

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(50)).await;
                drain(&worker.queue, &worker.database);
            }
        });

        it
    }
}

#[async_trait::async_trait]
impl AuditWriter for DatabaseAuditWriter {
    async fn write(&self, payload: AuditPayload) {
        self.queue
            .push(audit_model_to_active_model(model::audits::Model {
                id: Uuid::now_v7(),
                audit_type: payload.audit_type(),
                payload: serde_json::to_value(payload).unwrap(),
                created_at: Utc::now().into(),
            }));
    }
}

fn drain(queue: &SegQueue<AuditActiveModel>, database: &DatabaseConnection) {
    let mut buffer = Vec::new();
    while let Some(pat) = queue.pop() {
        buffer.push(pat);
    }

    if !buffer.is_empty() {
        let db = database.clone();
        tokio::spawn(async move {
            Audits::insert_many_audits(buffer, &db).await.unwrap();
        });
    }
}
