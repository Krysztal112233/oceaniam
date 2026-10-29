//! PGMQ worker triggers. Queue SQL belongs here, not in business handlers or the runtime.
//!
//! Like the generic `pgmq` helpers, these functions have no SeaORM entity: the queue lives in
//! the extension's schema. The read count is a claim generation, fencing stale renewals/acks.

use sea_orm::{FromQueryResult, Statement};
use serde_json::json;

use crate::{
    error::Error,
    helper::{SafeTransactionConnectionTrait, pgmq},
};

pub const WORKER_QUEUE: &str = "worker_jobs";

#[derive(Debug, FromQueryResult)]
pub struct WorkerJob {
    pub msg_id: i64,
    pub read_ct: i32,
    pub message: serde_json::Value,
}

/// Publishes a manual, business-event, or delayed trigger; it can join a business transaction.
#[tracing::instrument(level = "info", skip_all, fields(otel.kind = "internal", %worker_name))]
pub async fn enqueue(
    worker_name: &str,
    delay_seconds: i32,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<i64, Error> {
    pgmq::send(
        WORKER_QUEUE,
        json!({ "worker": worker_name }),
        delay_seconds,
        database,
    )
    .await
}

/// Claims due triggers. Callers must not cancel the read: the claim may already be committed.
#[tracing::instrument(level = "debug", skip_all, fields(otel.kind = "internal"))]
pub async fn read(
    visibility_timeout_seconds: i32,
    quantity: i32,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<Vec<WorkerJob>, Error> {
    pgmq::read(WORKER_QUEUE, visibility_timeout_seconds, quantity, database).await
}

/// Extends only this claim's lease. A re-delivered message belongs to its newer read count.
#[tracing::instrument(level = "debug", skip_all, fields(otel.kind = "internal", msg_id = job.msg_id))]
pub async fn renew(
    job: &WorkerJob,
    visibility_timeout_seconds: i32,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<bool, Error> {
    let result = database
        .execute_raw(Statement::from_sql_and_values(
            database.get_database_backend(),
            "UPDATE pgmq.q_worker_jobs
         SET vt = clock_timestamp() + make_interval(secs => $3::integer)
         WHERE msg_id = $1 AND read_ct = $2",
            [
                job.msg_id.into(),
                job.read_ct.into(),
                visibility_timeout_seconds.into(),
            ],
        ))
        .await?;
    Ok(result.rows_affected() == 1)
}

/// Acknowledges only this claim; an old consumer cannot delete a newer consumer's message.
#[tracing::instrument(level = "debug", skip_all, fields(otel.kind = "internal", msg_id = job.msg_id))]
pub async fn ack(
    job: &WorkerJob,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<bool, Error> {
    let result = database
        .execute_raw(Statement::from_sql_and_values(
            database.get_database_backend(),
            "DELETE FROM pgmq.q_worker_jobs WHERE msg_id = $1 AND read_ct = $2",
            [job.msg_id.into(), job.read_ct.into()],
        ))
        .await?;
    Ok(result.rows_affected() == 1)
}

/// Archives malformed/unknown triggers or exhausted retries, without stealing a newer claim.
#[tracing::instrument(level = "info", skip_all, fields(otel.kind = "internal", msg_id = job.msg_id))]
pub async fn archive(
    job: &WorkerJob,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<bool, Error> {
    let row = database
        .query_one_raw(Statement::from_sql_and_values(
            database.get_database_backend(),
            "WITH claimed AS MATERIALIZED (
            SELECT msg_id FROM pgmq.q_worker_jobs
            WHERE msg_id = $1 AND read_ct = $2 FOR UPDATE
         )
         SELECT pgmq.archive('worker_jobs', msg_id) AS archived FROM claimed",
            [job.msg_id.into(), job.read_ct.into()],
        ))
        .await?;
    match row {
        Some(row) => Ok(row.try_get("", "archived")?),
        None => Ok(false),
    }
}
