use std::sync::Arc;

use async_trait::async_trait;
use oceaniam_common::error::Error;
use oceaniam_database::{
    helper::audit_summary_by_application::AuditSummaryByApplicationHelper,
    view::prelude::AuditSummaryByApplication,
};

use crate::runtime::{REGISTERED_WORKERS, Worker, WorkerContext, WorkerFactory, WorkerRef};

pub struct RefreshAuditSummaryByApplicationWorker;

fn refresh_audit_summary_by_application_worker() -> WorkerRef {
    Arc::new(RefreshAuditSummaryByApplicationWorker)
}

#[linkme::distributed_slice(REGISTERED_WORKERS)]
static REFRESH_AUDIT_SUMMARY_BY_APPLICATION_REGISTRATION: WorkerFactory =
    refresh_audit_summary_by_application_worker;

#[async_trait]
impl Worker for RefreshAuditSummaryByApplicationWorker {
    fn name(&self) -> &'static str {
        "refresh_audit_summary_by_application"
    }

    /// Runs once every five minutes.
    fn cron(&self) -> &'static str {
        "0 */5 * * * * *"
    }

    async fn run(&self, context: &WorkerContext) -> Result<(), Error> {
        AuditSummaryByApplication::refresh(&context.database).await
    }
}
