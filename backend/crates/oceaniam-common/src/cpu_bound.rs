use tracing::{Instrument, Span, dispatcher};

use crate::consts;

/// Runs CPU-intensive work on Tokio's blocking pool while preserving tracing context.
///
/// The global CPU semaphore is acquired inside `queue_span`. Once capacity is available, the
/// current dispatcher and parent span are restored on the blocking thread before invoking `task`.
pub async fn run_cpu_bound<T, F>(queue_span: Span, task: F) -> Result<T, tokio::task::JoinError>
where
    T: Send + 'static,
    F: FnOnce(Span) -> T + Send + 'static,
{
    let permit = consts::MAX_CPU_BOUND_SEMAPHORE
        .acquire()
        .instrument(queue_span)
        .await
        .expect("cpu-bound semaphore should not be closed");
    let parent = Span::current();
    let dispatch = dispatcher::get_default(Clone::clone);

    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        dispatcher::with_default(&dispatch, || task(parent))
    })
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use tokio::sync::oneshot;
    use tracing::Span;

    use super::*;

    // NOTE: AI-generated test
    #[tokio::test]
    async fn cancelled_caller_does_not_release_permit_before_blocking_task_finishes() {
        let capacity = consts::MAX_CPU_BOUND_SEMAPHORE.available_permits();
        assert!(capacity > 0, "CPU semaphore must have positive capacity");

        let reserved = if capacity > 1 {
            Some(
                consts::MAX_CPU_BOUND_SEMAPHORE
                    .acquire_many((capacity - 1) as u32)
                    .await
                    .expect("reserve semaphore permits"),
            )
        } else {
            None
        };

        let (first_started_tx, first_started_rx) = oneshot::channel();
        let (first_release_tx, first_release_rx) = mpsc::channel();
        let first = tokio::spawn(run_cpu_bound(Span::none(), move |_| {
            first_started_tx.send(()).expect("report first task start");
            first_release_rx.recv().expect("release first task");
        }));

        first_started_rx.await.expect("first task should start");
        first.abort();
        assert!(
            first
                .await
                .expect_err("caller task should be cancelled")
                .is_cancelled(),
            "aborting the caller should cancel only its join future"
        );
        assert_eq!(
            consts::MAX_CPU_BOUND_SEMAPHORE.available_permits(),
            0,
            "the detached blocking task must retain its permit"
        );

        let (second_started_tx, mut second_started_rx) = oneshot::channel();
        let second = tokio::spawn(run_cpu_bound(Span::none(), move |_| {
            second_started_tx
                .send(())
                .expect("report second task start");
        }));
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut second_started_rx)
                .await
                .is_err(),
            "a second CPU task must remain queued while the detached task is running"
        );

        first_release_tx.send(()).expect("release first task");
        tokio::time::timeout(Duration::from_secs(2), &mut second_started_rx)
            .await
            .expect("second task should start after the first finishes")
            .expect("second task start signal");
        second
            .await
            .expect("second caller task")
            .expect("second blocking task");

        drop(reserved);
    }
}
