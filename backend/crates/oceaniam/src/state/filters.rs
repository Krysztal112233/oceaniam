use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use log::error;
use oceaniam_common::error::Error;
use oceaniam_database::{
    helper::{applications::ApplicationHelper, applications_secrets::ApplicationSecretsHelper},
    model::prelude::{ApplicationSecrets, Applications},
};
use oceaniam_filter::Filter;
use sea_orm::DatabaseConnection;

type RefreshFuture = Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>>;
type RefreshFn<'a> = Arc<dyn Fn(Filter<'a>) -> RefreshFuture + Send + Sync + 'static>;

#[derive(Clone)]
pub struct AutomaticRefreshFilters<'a> {
    filter: Filter<'a>,
    refresh: RefreshFn<'a>,

    dirty: Arc<AtomicBool>,
    dropped: Arc<AtomicBool>,

    alive: Arc<()>,
}

impl<'a> std::ops::Deref for AutomaticRefreshFilters<'a> {
    type Target = Filter<'a>;

    fn deref(&self) -> &Self::Target {
        &self.filter
    }
}

impl std::fmt::Debug for AutomaticRefreshFilters<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutomaticRefreshFilters")
            .field("dirty", &self.dirty.load(Ordering::Relaxed))
            .field("dropped", &self.dropped.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl Drop for AutomaticRefreshFilters<'_> {
    fn drop(&mut self) {
        if Arc::strong_count(&self.alive) == 1 {
            self.dropped.store(true, Ordering::Relaxed);
        }
    }
}

impl AutomaticRefreshFilters<'_> {
    fn new<'a: 'static>(filter: Filter<'a>, refresh: RefreshFn<'a>) -> AutomaticRefreshFilters<'a> {
        let dropped = Arc::new(AtomicBool::new(false));
        let dirty = Arc::new(AtomicBool::new(true));
        let alive = Arc::new(());

        let instance = AutomaticRefreshFilters {
            filter,
            refresh,
            dirty,
            dropped,
            alive,
        };

        let dropped_for_task = instance.dropped.clone();
        let dirty_for_task = instance.dirty.clone();
        let filter_for_task = instance.filter.clone();
        let refresh_for_task = instance.refresh.clone();
        let alive_for_task = Arc::downgrade(&instance.alive);

        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_millis(100));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tick.tick().await;

                if dropped_for_task.load(Ordering::Relaxed) {
                    break;
                }

                if alive_for_task.upgrade().is_none() {
                    dropped_for_task.store(true, Ordering::Relaxed);
                    break;
                }

                let should_refresh = dirty_for_task.swap(false, Ordering::AcqRel);
                if !should_refresh {
                    continue;
                }

                // Double-check right before refresh to avoid doing work after being dropped.
                if dropped_for_task.load(Ordering::Relaxed) {
                    break;
                }

                if let Err(e) = (refresh_for_task)(filter_for_task.clone()).await {
                    error!("failed to refresh filter: {e}");
                    dirty_for_task.store(true, Ordering::Release);
                }
            }
        });

        instance
    }
}

impl<'a> AutomaticRefreshFilters<'a> {
    pub fn mark(&self) {
        self.dirty.store(true, Ordering::Release);
    }
}

#[derive(Debug, Clone)]
pub struct ManagedFilters<'a> {
    application_id_filter: AutomaticRefreshFilters<'a>,
    secret_id_filter: AutomaticRefreshFilters<'a>,
    secret_filter: AutomaticRefreshFilters<'a>,
}

impl ManagedFilters<'_> {
    pub fn new<'a: 'static>(database: DatabaseConnection) -> ManagedFilters<'a> {
        let refresh_application_id_filter: RefreshFn<'a> = {
            let database = database.clone();
            Arc::new(move |filter: Filter<'a>| {
                let database = database.clone();
                Box::pin(async move {
                    let keys = Applications::get_all_application_ids(&database).await?;
                    filter.replace_from(&keys).await?;
                    Ok(())
                })
            })
        };

        let refresh_secret_id_filter: RefreshFn<'a> = {
            let database = database.clone();
            Arc::new(move |filter: Filter<'a>| {
                let database = database.clone();
                Box::pin(async move {
                    let keys = ApplicationSecrets::get_all_secret_ids(&database).await?;
                    filter.replace_from(&keys).await?;
                    Ok(())
                })
            })
        };

        let refresh_secret_filter: RefreshFn<'a> = {
            let database = database.clone();
            Arc::new(move |filter: Filter<'a>| {
                let database = database.clone();
                Box::pin(async move {
                    let keys = ApplicationSecrets::get_all_secrets(&database).await?;
                    filter.replace_from(&keys).await?;
                    Ok(())
                })
            })
        };

        ManagedFilters {
            application_id_filter: AutomaticRefreshFilters::new(
                Filter::new(),
                refresh_application_id_filter,
            ),
            secret_id_filter: AutomaticRefreshFilters::new(Filter::new(), refresh_secret_id_filter),
            secret_filter: AutomaticRefreshFilters::new(Filter::new(), refresh_secret_filter),
        }
    }
}

impl<'a> ManagedFilters<'a> {
    pub fn application_id_filter(&self) -> &AutomaticRefreshFilters<'a> {
        &self.application_id_filter
    }

    pub fn secret_id_filter(&self) -> &AutomaticRefreshFilters<'a> {
        &self.secret_id_filter
    }

    pub fn secret_filter(&self) -> &AutomaticRefreshFilters<'a> {
        &self.secret_filter
    }
}
