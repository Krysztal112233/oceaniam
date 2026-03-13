use std::{fmt::Debug, sync::Arc};

mod db_writer;

use db_writer::DatabaseAuditWriter;
use oceaniam_audit::types::AuditPayload;
use sea_orm::DatabaseConnection;

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
