//! Starts one queue consumer and, optionally, a cron-to-queue producer.

use std::{collections::HashMap, fmt, marker::PhantomData, str::FromStr, sync::Arc};

use cron::Schedule;
use sea_orm::DatabaseConnection;
use tokio::{sync::watch, task::JoinHandle};
use tracing::warn;

use crate::{consumer, scheduler, worker::WorkerRegistration};

pub trait WorkerRuntimeError: fmt::Display + Send + Sync {
    fn internal(msg: String) -> Self;
}

pub(crate) type Registrations<Ctx, E> = HashMap<String, WorkerRegistration<Ctx, E>>;

pub struct WorkerRuntime<Ctx, E>
where
    Ctx: Clone + Send + Sync + 'static,
    E: WorkerRuntimeError + 'static,
{
    context: Ctx,
    /// PostgreSQL pool used for PGMQ transport, separate from the business context.
    queue_database: DatabaseConnection,
    workers: Registrations<Ctx, E>,
}

impl<Ctx, E> WorkerRuntime<Ctx, E>
where
    Ctx: Clone + Send + Sync + 'static,
    E: WorkerRuntimeError + 'static,
{
    pub fn new(
        context: Ctx,
        workers: Registrations<Ctx, E>,
        queue_database: DatabaseConnection,
    ) -> Self {
        Self {
            context,
            queue_database,
            workers,
        }
    }

    pub fn start(self) -> Result<WorkerRuntimeController<E>, E> {
        // Validate everything before spawning, so an invalid registration leaves no orphan tasks.
        let mut schedules = Vec::new();
        for (name, registration) in &self.workers {
            if name != registration.worker.name() {
                return Err(E::internal(format!(
                    "worker registration name mismatch: {name}"
                )));
            }
            if let Some(expression) = registration.cron {
                let schedule = Schedule::from_str(expression).map_err(|error| {
                    E::internal(format!("invalid cron for worker `{name}`: {error}"))
                })?;
                schedules.push((name.clone(), schedule));
            }
        }

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let mut handles = Vec::new();
        if self.workers.is_empty() {
            warn!("no workers registered");
        } else {
            handles.push(tokio::spawn(consumer::run(
                self.queue_database.clone(),
                self.context,
                Arc::new(self.workers),
                shutdown_rx.clone(),
            )));
        }
        if !schedules.is_empty() {
            handles.push(tokio::spawn(scheduler::run(
                self.queue_database,
                schedules,
                shutdown_rx,
            )));
        }

        Ok(WorkerRuntimeController {
            handles,
            shutdown_tx,
            _phantom: PhantomData,
        })
    }
}

pub struct WorkerRuntimeController<E> {
    handles: Vec<JoinHandle<()>>,
    shutdown_tx: watch::Sender<bool>,
    _phantom: PhantomData<E>,
}

impl<E: WorkerRuntimeError> WorkerRuntimeController<E> {
    /// Stops publishing/claiming and waits for every in-flight handler, including its ack.
    pub async fn shutdown(self) -> Result<(), E> {
        self.shutdown_tx.send_replace(true);
        let mut failure = None;
        for handle in self.handles {
            if let Err(error) = handle.await {
                failure = Some(E::internal(format!("worker runtime task aborted: {error}")));
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
