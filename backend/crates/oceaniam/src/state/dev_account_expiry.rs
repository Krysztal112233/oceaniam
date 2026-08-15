//! pgmq-backed development-account expiration consumer.
//!
//! A delayed pgmq message is enqueued in the same transaction that creates a development
//! account; the message delay **is** the timer (the message becomes visible exactly at
//! `expires_at`). This module polls the queue inside the API process — deletion reuses
//! [`crate::state::applications::ApplicationUsers::delete_user_in_tx`], which performs moka
//! cache eviction, so running the consumer in a stateless worker would leave a stale-auth
//! window (the credential cache TTL is 30 minutes).
//!
//! Acknowledgement semantics:
//! - success → `pgmq.delete`;
//! - subject already gone (manual deletion raced the timer) → treated as success, `pgmq.delete`;
//! - transient error (DB jitter, etc.) → no ack; the message reappears after the visibility
//!   timeout;
//! - `read_ct >= 10` (or a malformed payload, which can never succeed) → `pgmq.archive`
//!   (dead letter), the message is not processed.
//!
//! `pgmq.read` uses `FOR UPDATE SKIP LOCKED`, so multiple API replicas can run this loop
//! concurrently.

use std::future::Future;
use std::time::Duration;

use axum::http::StatusCode;
use oceaniam_audit::types::{AuditPayload, DevAccountExpiredPayload};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, FromQueryResult, Statement};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::Error;
use crate::state::AppState;

/// pgmq queue carrying development-account expiration messages.
pub const DEV_ACCOUNT_EXPIRATION_QUEUE: &str = "dev_account_expiration";

/// Visibility timeout (seconds) applied to claimed messages; unacked messages reappear after
/// this window.
const VISIBILITY_TIMEOUT_SECONDS: i32 = 60;

/// Maximum number of messages claimed per `pgmq.read` call.
const READ_BATCH_SIZE: i32 = 10;

/// Messages read this many times are considered poisoned and archived instead of processed.
const MAX_READ_COUNT: i32 = 10;

/// Interruptible idle sleep between polls when the queue is empty. This is the
/// only shutdown checkpoint — an in-flight `pgmq.read` is never cancelled because pgmq claims
/// the message server-side the instant the read executes (`read_ct += 1`, VT set), and
/// cancelling would burn a retry and delay redelivery by one visibility timeout.
const IDLE_SLEEP: Duration = Duration::from_secs(1);

/// Upper bound for the exponential backoff applied after consecutive `pgmq.read` failures
/// (e.g. a database where the migration has not been applied yet), so a persistently broken
/// environment does not produce one error log per second forever.
const MAX_READ_ERROR_BACKOFF: Duration = Duration::from_secs(60);

/// Payload enqueued at dev-account creation time.
///
/// `application_id` is included because deletion needs it for cache eviction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevAccountExpirationMessage {
    pub subject_id: Uuid,
    pub application_id: Uuid,
}

/// One row returned by `pgmq.read`.
#[derive(Clone, Debug, FromQueryResult)]
pub struct DevAccountExpirationRow {
    pub msg_id: i64,
    pub read_ct: i32,
    pub message: serde_json::Value,
}

/// Claims a batch of due expiration messages from the queue.
#[tracing::instrument(
    level = "info",
    name = "dev_account_expiry.read_batch",
    skip_all,
    fields(otel.kind = "internal")
)]
pub async fn read_expiration_batch(
    database: &DatabaseConnection,
) -> Result<Vec<DevAccountExpirationRow>, Error> {
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT msg_id, read_ct, message FROM pgmq.read($1, $2, $3)",
        [
            DEV_ACCOUNT_EXPIRATION_QUEUE.into(),
            VISIBILITY_TIMEOUT_SECONDS.into(),
            READ_BATCH_SIZE.into(),
        ],
    );

    let rows = database
        .query_all_raw(stmt)
        .await?
        .into_iter()
        .map(|r| DevAccountExpirationRow::from_query_result(&r, ""))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Deletes (acks) a message from the queue.
async fn pgmq_delete(database: &DatabaseConnection, msg_id: i64) -> Result<(), Error> {
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT pgmq.delete($1, $2)",
        [DEV_ACCOUNT_EXPIRATION_QUEUE.into(), msg_id.into()],
    );
    database.query_one_raw(stmt).await?;

    Ok(())
}

/// Moves a message to the dead-letter archive (`pgmq.a_dev_account_expiration`).
async fn pgmq_archive(database: &DatabaseConnection, msg_id: i64) -> Result<(), Error> {
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT pgmq.archive($1, $2)",
        [DEV_ACCOUNT_EXPIRATION_QUEUE.into(), msg_id.into()],
    );
    database.query_one_raw(stmt).await?;

    Ok(())
}

fn is_not_found(error: &Error) -> bool {
    matches!(
        error,
        Error::CustomMessage { code, .. } if *code == StatusCode::NOT_FOUND.as_u16()
    )
}

/// Processes a single expiration message: permanently deletes the account and acknowledges
/// the message.
///
/// Factored out of the poll loop so integration tests can invoke it directly. Idempotent:
/// deleting an already-gone account is success, so invoking this twice for the same message is
/// a no-op.
#[tracing::instrument(
    level = "info",
    name = "dev_account_expiry.process_message",
    skip_all,
    fields(otel.kind = "internal", msg_id = row.msg_id, read_ct = row.read_ct)
)]
pub async fn process_expiration_message(
    state: &AppState,
    row: &DevAccountExpirationRow,
) -> Result<(), Error> {
    let payload = serde_json::from_value::<DevAccountExpirationMessage>(row.message.clone());

    // Poison handling: a message that exceeded the retry budget, or one whose payload can never
    // deserialize, is archived (dead-lettered) instead of processed.
    let poisoned = row.read_ct >= MAX_READ_COUNT || payload.is_err();
    if poisoned {
        warn!(
            msg_id = row.msg_id,
            read_ct = row.read_ct,
            "archiving poisoned dev account expiration message"
        );
        pgmq_archive(&state.database, row.msg_id).await?;
        return Ok(());
    }

    let DevAccountExpirationMessage {
        subject_id,
        application_id,
    } = payload.expect("payload validity checked above");

    let result = async {
        let users = state
            .applications
            .get_application_users(application_id)
            .await?;
        users
            .delete_user_in_tx(application_id, subject_id, &state.database)
            .await
    }
    .await;

    match result {
        Ok(()) => {
            info!(
                %subject_id,
                %application_id,
                "dev account expired and deleted"
            );

            state
                .auditing
                .write(AuditPayload::from(DevAccountExpiredPayload {
                    application_id,
                    subject_id,
                }))
                .await;

            pgmq_delete(&state.database, row.msg_id).await?;
        }
        Err(error) if is_not_found(&error) => {
            // Manual deletion raced the timer (or the whole application is gone); an
            // already-gone account counts as success.
            info!(
                %subject_id,
                %application_id,
                "dev account already gone, acknowledging expiration message"
            );
            pgmq_delete(&state.database, row.msg_id).await?;
        }
        Err(error) => {
            // Transient failure: do not ack; the message reappears after the visibility
            // timeout.
            error!(
                %subject_id,
                %application_id,
                error = %error,
                "failed to delete expired dev account; leaving message for redelivery"
            );
            return Err(error);
        }
    }

    Ok(())
}

/// Runs the consumer loop until `shutdown` resolves.
///
/// Spawned once per API process at bootstrap; safe to run on multiple replicas because
/// `pgmq.read` claims messages with `FOR UPDATE SKIP LOCKED`.
pub async fn run(state: AppState, shutdown: impl Future<Output = ()> + Send) {
    info!("dev account expiration consumer started");

    let mut shutdown = std::pin::pin!(shutdown);
    let mut consecutive_read_errors: u32 = 0;

    loop {
        match read_expiration_batch(&state.database).await {
            Ok(rows) if rows.is_empty() => {
                consecutive_read_errors = 0;
                debug!("no due dev account expiration messages");
            }
            Ok(rows) => {
                consecutive_read_errors = 0;
                for row in &rows {
                    if let Err(error) = process_expiration_message(&state, row).await {
                        error!(
                            msg_id = row.msg_id,
                            error = %error,
                            "dev account expiration message processing failed"
                        );
                    }
                }
                // Drain immediately: more messages may already be visible.
                continue;
            }
            Err(error) => {
                consecutive_read_errors += 1;
                error!(error = %error, consecutive_read_errors, "dev account expiration read failed");
            }
        }

        // Exponential backoff on consecutive read failures, capped at MAX_READ_ERROR_BACKOFF;
        // empty reads keep the base idle sleep.
        let backoff = if consecutive_read_errors == 0 {
            IDLE_SLEEP
        } else {
            IDLE_SLEEP
                .saturating_mul(1u32 << consecutive_read_errors.min(6))
                .min(MAX_READ_ERROR_BACKOFF)
        };

        tokio::select! {
            _ = &mut shutdown => {
                info!("dev account expiration consumer shutting down");
                break;
            }
            _ = tokio::time::sleep(backoff) => {}
        }
    }
}
