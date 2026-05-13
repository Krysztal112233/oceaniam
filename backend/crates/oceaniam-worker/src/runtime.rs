use std::{
    collections::HashMap,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use chrono::Utc;
use cron::Schedule;
use linkme::distributed_slice;
use oceaniam::error::Error;
use sea_orm::DatabaseConnection;
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::{debug, error, info, warn};

pub type WorkerRef = Arc<dyn Worker>;

#[derive(Clone)]
pub struct WorkerContext {
    pub database: DatabaseConnection,
}

#[async_trait]
pub trait Worker: Send + Sync {
    fn name(&self) -> &'static str;
    fn cron(&self) -> &'static str;

    async fn run(&self, context: &WorkerContext) -> Result<(), Error>;
}

pub type WorkerFactory = fn() -> WorkerRef;

#[distributed_slice]
pub static REGISTERED_WORKERS: [WorkerFactory];

pub struct WorkerRuntime {
    context: WorkerContext,
    workers: HashMap<String, WorkerRef>,
}

impl WorkerRuntime {
    pub fn new(context: WorkerContext) -> Result<Self, Error> {
        let workers = REGISTERED_WORKERS
            .iter()
            .map(|factory| {
                let worker = factory();
                let name = worker.name().to_owned();
                (name, worker)
            })
            .collect();

        Ok(Self { context, workers })
    }

    pub async fn run(self) -> Result<(), Error> {
        let (shutdown_tx, _) = broadcast::channel(1);

        let handles = self
            .workers
            .iter()
            .map(|(name, worker)| {
                spawn_worker_loop(
                    name.clone(),
                    worker.clone(),
                    self.context.clone(),
                    shutdown_tx.subscribe(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        if handles.is_empty() {
            warn!("no worker schedules started");
        }

        shutdown_signal().await;
        debug!("worker runtime shutting down");

        let _ = shutdown_tx.send(());

        for handle in handles {
            handle.await.map_err(|err| Error::Internal {
                msg: format!("worker scheduler task aborted unexpectedly: {err}"),
                location: snafu::location!(),
            })?;
        }

        Ok(())
    }
}

fn spawn_worker_loop(
    name: String,
    worker: WorkerRef,
    context: WorkerContext,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<JoinHandle<()>, Error> {
    let cron = worker.cron();
    let schedule = Schedule::from_str(cron).map_err(|err| Error::Internal {
        msg: format!("invalid cron for worker `{name}`: {err}"),
        location: snafu::location!(),
    })?;

    Ok(tokio::spawn(async move {
        let running = Arc::new(AtomicBool::new(false));

        info!(worker = %name, cron, "worker schedule started");

        loop {
            let Some(next_tick) = schedule.upcoming(Utc).next() else {
                warn!(worker = %name, "worker schedule exhausted; stopping");
                break;
            };

            let wait_for = next_tick
                .signed_duration_since(Utc::now())
                .to_std()
                .unwrap_or_default();

            tokio::select! {
                _ = tokio::time::sleep(wait_for) => {}
                result = shutdown_rx.recv() => {
                    if let Err(err) = result {
                        debug!(worker = %name, error = %err, "shutdown channel closed");
                    }

                    break;
                }
            }

            if running.swap(true, Ordering::AcqRel) {
                warn!(worker = %name, "worker still running; skipping overlapping tick");
                continue;
            }

            let running = running.clone();
            let name = name.clone();
            let worker = worker.clone();
            let context = context.clone();

            tokio::spawn(async move {
                info!(worker = %name, "worker execution started");

                if let Err(err) = worker.run(&context).await {
                    error!(worker = %name, error = %err, "worker execution failed");
                } else {
                    info!(worker = %name, "worker execution completed");
                }

                running.store(false, Ordering::Release);
            });
        }

        info!(worker = %name, "worker schedule stopped");
    }))
}

async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate =
        signal(SignalKind::terminate()).expect("failed to install SIGTERM signal handler");
    let mut interrupt =
        signal(SignalKind::interrupt()).expect("failed to install SIGINT signal handler");

    tokio::select! {
        _ = terminate.recv() => debug!("received SIGTERM, starting worker shutdown"),
        _ = interrupt.recv() => debug!("received SIGINT, starting worker shutdown"),
    }
}

#[cfg(test)]
mod tests {
    use cron::Schedule;

    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn test_all_registered_worker_crons_compile() {
        for worker in REGISTERED_WORKERS.iter().map(|it| it()) {
            Schedule::from_str(worker.cron())
                .unwrap_or_else(|err| panic!("invalid cron for worker `{}`: {err}", worker.name()));
        }
    }
}
