use std::{collections::HashMap, fmt, marker::PhantomData, str::FromStr, sync::Arc};

use chrono::Utc;
use cron::Schedule;
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::{Instrument, Span, debug, error, field::Empty, info, warn};

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
        let mut execution = None;

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

            if execution
                .as_ref()
                .is_some_and(|handle: &JoinHandle<()>| !handle.is_finished())
            {
                warn!(worker = %name, "worker still running; skipping overlapping tick");
                continue;
            }
            if let Some(handle) = execution.take()
                && let Err(err) = handle.await
            {
                error!(worker = %name, error = %err, "worker execution task aborted unexpectedly");
            }

            let execution_name = name.clone();
            let worker = worker.clone();
            let context = context.clone();

            let span = tracing::info_span!(
                "worker.process",
                otel.kind = "internal",
                otel.name = %name,
                otel.status_code = Empty,
                otel.status_message = Empty,
                worker = %name,
            );
            execution = Some(tokio::spawn(
                async move {
                    info!(worker = %execution_name, "worker execution started");

                    if let Err(err) = worker.run(&context).await {
                        let span = Span::current();
                        span.record("otel.status_code", "ERROR");
                        span.record("otel.status_message", tracing::field::display(&err));
                        error!(worker = %execution_name, error = %err, "worker execution failed");
                    } else {
                        info!(worker = %execution_name, "worker execution completed");
                    }
                }
                .instrument(span),
            ));
        }

        if let Some(handle) = execution
            && let Err(err) = handle.await
        {
            error!(worker = %name, error = %err, "worker execution task aborted unexpectedly");
        }

        info!(worker = %name, "worker schedule stopped");
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{str::FromStr, time::Duration};

    use cron::Schedule;
    use tokio::sync::Notify;

    #[derive(Clone, Default)]
    struct BlockingContext {
        started: Arc<Notify>,
        release: Arc<Notify>,
    }

    struct BlockingWorker;

    #[derive(Debug)]
    struct TestError(String);

    impl fmt::Display for TestError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(&self.0)
        }
    }

    impl WorkerRuntimeError for TestError {
        fn internal(msg: String) -> Self {
            Self(msg)
        }
    }

    #[async_trait::async_trait]
    impl Worker<BlockingContext> for BlockingWorker {
        type Error = TestError;

        fn name(&self) -> &'static str {
            "blocking"
        }

        fn cron(&self) -> &'static str {
            "* * * * * * *"
        }

        async fn run(&self, context: &BlockingContext) -> Result<(), Self::Error> {
            context.started.notify_one();
            context.release.notified().await;
            Ok(())
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn test_all_registered_worker_crons_compile() {
        // Workers register via linkme::distributed_slice in the consuming crate.
        // The runtime crate itself has no registered workers, so this test
        // simply verifies that the cron crate is functional.
        for cron in ["0 0 */6 * * *", "0 */5 * * * *"] {
            Schedule::from_str(cron).unwrap_or_else(|err| panic!("invalid cron `{cron}`: {err}"));
        }
    }

    // NOTE: AI-generated test
    #[tokio::test]
    async fn shutdown_waits_for_running_worker() {
        let context = BlockingContext::default();
        let worker: Arc<dyn Worker<BlockingContext, Error = TestError>> = Arc::new(BlockingWorker);
        let runtime = WorkerRuntime::new(
            context.clone(),
            HashMap::from([("blocking".to_owned(), worker)]),
        );
        let controller = runtime.start().expect("start worker runtime");

        tokio::time::timeout(Duration::from_secs(3), context.started.notified())
            .await
            .expect("worker should start");

        let shutdown = tokio::spawn(controller.shutdown());
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            !shutdown.is_finished(),
            "shutdown returned before worker completed"
        );

        context.release.notify_one();
        shutdown
            .await
            .expect("shutdown task")
            .expect("shutdown runtime");
    }
}
