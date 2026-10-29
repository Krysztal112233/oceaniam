//! One bounded consumer loop. Handler execution never originates in the cron producer.

use std::{sync::Arc, time::Duration};

use oceaniam_database::helper::worker_jobs::{self, WorkerJob};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use tokio::{sync::watch, task::JoinSet};
use tracing::{Span, error, field::Empty, info, warn};

use crate::runtime::{Registrations, WorkerRuntimeError};

const CONCURRENCY: usize = 4;
const VISIBILITY_SECONDS: i32 = 120;
const RENEW_INTERVAL: Duration = Duration::from_secs(30);
const MAX_ATTEMPTS: i32 = 10;
const IDLE_INTERVAL: Duration = Duration::from_secs(1);
const MAX_READ_BACKOFF: Duration = Duration::from_secs(60);

#[derive(Deserialize)]
struct Trigger {
    worker: String,
}

pub(crate) async fn run<Ctx, E>(
    queue_database: DatabaseConnection,
    context: Ctx,
    workers: Arc<Registrations<Ctx, E>>,
    mut shutdown: watch::Receiver<bool>,
) where
    Ctx: Clone + Send + Sync + 'static,
    E: WorkerRuntimeError + 'static,
{
    let mut executions = JoinSet::new();
    let mut read_errors: u32 = 0;
    info!("worker queue consumer started");

    while !*shutdown.borrow() {
        while let Some(result) = executions.try_join_next() {
            if let Err(error) = result {
                error!(%error, "worker execution aborted; trigger will be redelivered");
            }
        }
        if executions.len() == CONCURRENCY {
            tokio::select! {
                _ = shutdown.changed() => break,
                result = executions.join_next() => {
                    if let Some(Err(error)) = result {
                        error!(%error, "worker execution aborted; trigger will be redelivered");
                    }
                }
            }
            continue;
        }

        // Do not cancel an in-flight claim, even when shutdown arrives.
        match worker_jobs::read(VISIBILITY_SECONDS, 1, &queue_database).await {
            Ok(jobs) if !jobs.is_empty() => {
                read_errors = 0;
                for job in jobs {
                    executions.spawn(process(
                        job,
                        queue_database.clone(),
                        context.clone(),
                        workers.clone(),
                    ));
                }
                continue;
            }
            Ok(_) => read_errors = 0,
            Err(error) => {
                read_errors = read_errors.saturating_add(1);
                error!(%error, read_errors, "failed to read worker queue");
            }
        }

        let backoff = if read_errors == 0 {
            IDLE_INTERVAL
        } else {
            IDLE_INTERVAL
                .saturating_mul(1 << read_errors.min(6))
                .min(MAX_READ_BACKOFF)
        };
        tokio::select! {
            _ = shutdown.changed() => break,
            _ = tokio::time::sleep(backoff) => {}
        }
    }

    // No new claims; existing executions keep renewing their leases until completion.
    while let Some(result) = executions.join_next().await {
        if let Err(error) = result {
            error!(%error, "worker execution aborted during shutdown");
        }
    }
    info!("worker queue consumer stopped");
}

#[tracing::instrument(
    name = "worker.process",
    skip_all,
    fields(
        otel.kind = "internal", otel.name = Empty,
        otel.status_code = Empty, otel.status_message = Empty,
        worker = Empty, msg_id = job.msg_id, attempt = job.read_ct
    )
)]
async fn process<Ctx, E>(
    job: WorkerJob,
    queue_database: DatabaseConnection,
    context: Ctx,
    workers: Arc<Registrations<Ctx, E>>,
) where
    Ctx: Clone + Send + Sync + 'static,
    E: WorkerRuntimeError + 'static,
{
    let registration = serde_json::from_value::<Trigger>(job.message.clone())
        .ok()
        .and_then(|trigger| workers.get(&trigger.worker));
    let Some(registration) = registration else {
        warn!("archiving malformed or unknown worker trigger");
        archive(&job, &queue_database).await;
        return;
    };
    if job.read_ct > MAX_ATTEMPTS {
        warn!("worker claim budget exhausted (including crashes); archiving trigger");
        archive(&job, &queue_database).await;
        return;
    }
    let worker = &registration.worker;
    let name = worker.name();
    let span = Span::current();
    span.record("worker", name);
    span.record("otel.name", name);
    let execution = worker.run(&context);
    tokio::pin!(execution);
    let mut heartbeat =
        tokio::time::interval_at(tokio::time::Instant::now() + RENEW_INTERVAL, RENEW_INTERVAL);

    info!(worker = name, "worker execution started");
    let result = loop {
        tokio::select! {
            result = &mut execution => break result,
            _ = heartbeat.tick() => {
                match worker_jobs::renew(&job, VISIBILITY_SECONDS, &queue_database).await {
                    Ok(true) => {}
                    Ok(false) => {
                        warn!(worker = name, "worker claim lost; stopping stale execution");
                        return;
                    }
                    Err(error) => {
                        error!(worker = name, %error, "failed to renew claim; leaving trigger for retry");
                        return;
                    }
                }
            }
        }
    };

    match result {
        Ok(()) => match worker_jobs::ack(&job, &queue_database).await {
            Ok(true) => info!(worker = name, "worker execution completed"),
            Ok(false) => warn!(worker = name, "worker completed after losing its claim"),
            Err(error) => error!(worker = name, %error, "failed to ack worker; trigger may repeat"),
        },
        Err(error) => {
            span.record("otel.status_code", "ERROR");
            span.record("otel.status_message", tracing::field::display(&error));
            error!(worker = name, %error, "worker execution failed");
            if job.read_ct >= MAX_ATTEMPTS {
                archive(&job, &queue_database).await;
            }
            // Otherwise no ack: visibility timeout supplies the retry delay.
        }
    }
}

async fn archive(job: &WorkerJob, queue_database: &DatabaseConnection) {
    match worker_jobs::archive(job, queue_database).await {
        Ok(true) => warn!(
            msg_id = job.msg_id,
            "worker trigger archived; operator attention required"
        ),
        Ok(false) => warn!(
            msg_id = job.msg_id,
            "worker claim already lost before archive"
        ),
        Err(error) => error!(msg_id = job.msg_id, %error, "failed to archive worker trigger"),
    }
}
