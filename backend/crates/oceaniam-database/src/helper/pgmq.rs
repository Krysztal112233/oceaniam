//! Thin wrappers over the `pgmq` extension functions used by the development-account
//! expiration flow.
//!
//! pgmq is a deployment prerequisite (the extension and the queues are created by
//! migrations), so calling these helpers requires no setup beyond the migrated schema. All
//! functions take `&impl SafeTransactionConnectionTrait` like every other helper, so `send`
//! can join the caller's transaction and become visible atomically with the business write.

use sea_orm::{FromQueryResult, Statement};

use crate::{error::Error, helper::SafeTransactionConnectionTrait};

/// Enqueues `payload` onto `queue`, making it visible after `delay_seconds`.
///
/// Returns the message id assigned by pgmq. When called on a transaction, the message only
/// becomes visible if that transaction commits.
pub async fn send(
    queue: &str,
    payload: serde_json::Value,
    delay_seconds: i32,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<i64, Error> {
    let stmt = Statement::from_sql_and_values(
        database.get_database_backend(),
        "SELECT pgmq.send($1, $2, $3)",
        [queue.into(), payload.into(), delay_seconds.into()],
    );

    // `DbErr::Custom` (not `RecordNotFound`) keeps the caller's established 500 shape: the
    // `RecordNotFound` variant would be remapped to a 404 by the API error layer.
    let msg_id = database
        .query_one_raw(stmt)
        .await?
        .ok_or_else(|| {
            Error::from(sea_orm::DbErr::Custom(
                "pgmq.send returned no message id".to_owned(),
            ))
        })?
        .try_get("", "send")?;

    Ok(msg_id)
}

/// Claims up to `batch_size` due messages from `queue`, hiding them for
/// `visibility_timeout_seconds`.
///
/// `pgmq.read` claims messages with `FOR UPDATE SKIP LOCKED`, so multiple consumers can call
/// this concurrently. Unacknowledged messages reappear after the visibility timeout.
pub async fn read<R>(
    queue: &str,
    visibility_timeout_seconds: i32,
    batch_size: i32,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<Vec<R>, Error>
where
    R: FromQueryResult + Send,
{
    let stmt = Statement::from_sql_and_values(
        database.get_database_backend(),
        "SELECT msg_id, read_ct, message FROM pgmq.read($1, $2, $3)",
        [
            queue.into(),
            visibility_timeout_seconds.into(),
            batch_size.into(),
        ],
    );

    let rows = database
        .query_all_raw(stmt)
        .await?
        .iter()
        .map(|row| R::from_query_result(row, ""))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Deletes (acks) a message from the queue.
pub async fn delete(
    queue: &str,
    msg_id: i64,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<(), Error> {
    let stmt = Statement::from_sql_and_values(
        database.get_database_backend(),
        "SELECT pgmq.delete($1, $2)",
        [queue.into(), msg_id.into()],
    );
    database.query_one_raw(stmt).await?;

    Ok(())
}

/// Moves a message to the queue's dead-letter archive (`pgmq.a_<queue>`).
pub async fn archive(
    queue: &str,
    msg_id: i64,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<(), Error> {
    let stmt = Statement::from_sql_and_values(
        database.get_database_backend(),
        "SELECT pgmq.archive($1, $2)",
        [queue.into(), msg_id.into()],
    );
    database.query_one_raw(stmt).await?;

    Ok(())
}
