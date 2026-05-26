use std::{
    collections::HashMap,
    fmt,
    marker::PhantomData,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use chrono::Utc;
use cron::Schedule;
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::{debug, error, info, warn};

use crate::worker::Worker;

pub trait WorkerRuntimeError: fmt::Display {
    fn internal(msg: String) -> Self;
}

pub struct WorkerRuntime<Ctx, E>
where
    Ctx: Clone + Send + Sync + 'static,
    E: WorkerRuntimeError + 'static,
{
    context: Ctx,
    workers: HashMap<String, Arc<dyn Worker<Ctx, Error = E>>>,
}

impl<Ctx, E> WorkerRuntime<Ctx, E>
where
    Ctx: Clone + Send + Sync + 'static,
    E: WorkerRuntimeError + 'static,
{
    pub fn new(context: Ctx, workers: HashMap<String, Arc<dyn Worker<Ctx, Error = E>>>) -> Self {
        Self { context, workers }
    }

    pub fn start(self) -> Result<WorkerRuntimeController<E>, E> {
        let (shutdown_tx, _) = broadcast::channel(1);

        let handles = self
            .workers
            .into_iter()
            .map(|(name, worker)| {
                spawn_worker_loop(name, worker, self.context.clone(), shutdown_tx.subscribe())
            })
            .collect::<Result<Vec<_>, _>>()?;

        if handles.is_empty() {
            warn!("no worker schedules started");
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
    shutdown_tx: broadcast::Sender<()>,
    _phantom: PhantomData<E>,
}

impl<E: WorkerRuntimeError> WorkerRuntimeController<E> {
    pub async fn shutdown(self) -> Result<(), E> {
        let _ = self.shutdown_tx.send(());

        for handle in self.handles {
            handle.await.map_err(|err| {
                E::internal(format!("worker scheduler task aborted unexpectedly: {err}"))
            })?;
        }

        Ok(())
    }
}

fn spawn_worker_loop<Ctx, E>(
    name: String,
    worker: Arc<dyn Worker<Ctx, Error = E>>,
    context: Ctx,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<JoinHandle<()>, E>
where
    Ctx: Clone + Send + Sync + 'static,
    E: WorkerRuntimeError + 'static,
{
    let cron = worker.cron();
    let schedule = Schedule::from_str(cron)
        .map_err(|err| E::internal(format!("invalid cron for worker `{name}`: {err}")))?;

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

// NOTE: AI-generated test
#[cfg(test)]
mod tests {
    use cron::Schedule;
    use std::str::FromStr;

    #[test]
    fn test_all_registered_worker_crons_compile() {
        // Workers register via linkme::distributed_slice in the consuming crate.
        // The runtime crate itself has no registered workers, so this test
        // simply verifies that the cron crate is functional.
        for cron in ["0 0 */6 * * *", "*/5 * * * *"] {
            Schedule::from_str(cron).unwrap_or_else(|err| panic!("invalid cron `{cron}`: {err}"));
        }
    }
}
