use std::sync::Arc;

use chrono::Utc;
use crossbeam_queue::SegQueue;
use oceaniam_audit::types::AuditPayload;
use oceaniam_database::model::{self, prelude::Audits};
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};
use tokio::sync::oneshot;
use uuid::Uuid;

use super::AuditWriter;

type AuditActiveModel = model::audits::ActiveModel;

const AUDIT_BATCH_SIZE: usize = 2048;

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
                worker.compute_flush().await;
            }
        });

        it
    }

    async fn compute_flush(&self) {
        if self.queue.len() >= AUDIT_BATCH_SIZE {
            let mut buffer = Vec::new();
            while let Some(pat) = self.queue.pop() {
                buffer.push(pat);
            }

            flush(buffer, self.database.clone());
        }
    }
}

#[async_trait::async_trait]
impl AuditWriter for DatabaseAuditWriter {
    async fn write(&self, payload: AuditPayload) {
        self.queue.push(
            model::audits::Model {
                id: Uuid::now_v7(),
                audit_type: payload.audit_type(),
                payload: serde_json::to_value(payload).unwrap(),
                created_at: Utc::now().into(),
            }
            .into_active_model(),
        );
    }
}

fn flush(data: Vec<AuditActiveModel>, database: DatabaseConnection) {
    let (rx, tx) = oneshot::channel();
    rx.send(data).unwrap();

    tokio::spawn(async move {
        let data = tx.await.unwrap();

        Audits::insert_many(data).exec(&database).await.unwrap();
    });
}
