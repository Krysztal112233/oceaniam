use std::sync::Arc;

use async_trait::async_trait;
use linkme::distributed_slice;
use oceaniam_database::{
    helper::{
        oidc_authorization_codes::OidcAuthorizationCodesHelper,
        oidc_authorization_transactions::OidcAuthorizationTransactionsHelper,
    },
    model::prelude::{OidcAuthorizationCodes, OidcAuthorizationTransactions},
};
use oceaniam_worker_runtime::Worker;
use sea_orm::TransactionTrait;
use tracing::info;

use crate::{REGISTERED_WORKERS, WorkerContext, error::Error};

/// Hard-deletes expired OIDC authorization codes and authorization transactions once their
/// 24-hour forensics grace window has elapsed (protocol TTLs are 60s/10min; retention serves
/// audit correlation only). Codes go first, then transactions with no remaining code, all in
/// one database transaction; a transaction whose code is still inside its own grace window is
/// left for the next run.
struct OidcExpiredCleanupWorker;

#[async_trait]
impl Worker<WorkerContext> for OidcExpiredCleanupWorker {
    type Error = Error;

    fn name(&self) -> &'static str {
        "oidc_expired_cleanup"
    }

    fn cron(&self) -> &'static str {
        "0 17 * * * *"
    }

    async fn run(&self, context: &WorkerContext) -> Result<(), Error> {
        let transaction = context.database.begin().await?;

        let deleted_codes =
            OidcAuthorizationCodes::delete_expired_authorization_codes(&transaction).await?;
        let deleted_transactions =
            OidcAuthorizationTransactions::delete_expired_authorization_transactions(&transaction)
                .await?;

        transaction.commit().await?;

        info!(
            deleted_codes,
            deleted_transactions, "expired OIDC authorization rows cleaned up"
        );

        Ok(())
    }
}

#[distributed_slice(REGISTERED_WORKERS)]
static OIDC_EXPIRED_CLEANUP_WORKER: fn() -> crate::OceaniamWorkerRef =
    || Arc::new(OidcExpiredCleanupWorker);
