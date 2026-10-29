//! Single-process cron producer: wait for the next occurrence, then enqueue its trigger.

use chrono::{DateTime, Utc};
use cron::Schedule;
use oceaniam_database::helper::worker_jobs;
use sea_orm::DatabaseConnection;
use tokio::sync::watch;
use tracing::{error, info};

struct CronTrigger {
    name: String,
    schedule: Schedule,
    next: Option<DateTime<Utc>>,
}

pub(crate) async fn run(
    queue_database: DatabaseConnection,
    schedules: Vec<(String, Schedule)>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut triggers: Vec<_> = schedules
        .into_iter()
        .map(|(name, schedule)| CronTrigger {
            next: schedule.upcoming(Utc).next(),
            name,
            schedule,
        })
        .collect();
    info!("worker cron producer started");

    while !*shutdown.borrow() {
        let Some(next) = triggers.iter().filter_map(|trigger| trigger.next).min() else {
            break;
        };
        let wait = next
            .signed_duration_since(Utc::now())
            .to_std()
            .unwrap_or_default();
        tokio::select! {
            _ = shutdown.changed() => break,
            _ = tokio::time::sleep(wait) => {}
        }

        let now = Utc::now();
        for trigger in &mut triggers {
            if *shutdown.borrow() {
                break;
            }
            if trigger.next.is_none_or(|next| next > now) {
                continue;
            }
            match worker_jobs::enqueue(&trigger.name, 0, &queue_database).await {
                Ok(_) => {
                    info!(worker = %trigger.name, "worker trigger queued");
                    trigger.next = trigger.schedule.upcoming(Utc).next();
                }
                Err(error) => {
                    error!(worker = %trigger.name, %error, "failed to queue scheduled worker");
                    // Retry this publication without a busy loop or durable scheduling state.
                    trigger.next = Some(Utc::now() + chrono::Duration::seconds(5));
                }
            }
        }
    }
    info!("worker cron producer stopped");
}
