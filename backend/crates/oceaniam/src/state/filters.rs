use std::{
    future::Future,
    hash::BuildHasher,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use axum::http::StatusCode;
use itertools::Itertools;
use oceaniam_common::error::Error;
use oceaniam_database::{
    helper::{applications::ApplicationHelper, applications_secrets::ApplicationSecretsHelper},
    model::prelude::{ApplicationSecrets, Applications},
};
use parking_lot::RwLock;
use rapidhash::fast::SeedableState;
use sea_orm::DatabaseConnection;
use tracing::debug;
use xorf::{BinaryFuse32, Filter};

#[derive(Debug, Default)]
struct Metrics {
    hit: AtomicU64,
    miss: AtomicU64,
    cap: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct FilterWithMetrics<'a> {
    hasher: SeedableState<'a>,
    filter: Arc<RwLock<BinaryFuse32>>,
    metrics: Arc<Metrics>,
    is_empty: Arc<AtomicBool>,
    dirt: Arc<AtomicBool>,
}

impl FilterWithMetrics<'_> {
    pub fn new() -> Self {
        let metrics = Arc::new(Metrics::default());
        metrics.cap.store(1, Ordering::Relaxed);

        Self {
            hasher: SeedableState::random(),
            filter: Arc::new(RwLock::new(
                BinaryFuse32::try_from_iterator(vec![10].into_iter()).unwrap(),
            )),
            metrics,
            is_empty: Arc::new(AtomicBool::new(true)),
            dirt: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn exists<K>(&self, key: &K) -> bool
    where
        K: std::hash::Hash,
    {
        if self.is_empty.load(Ordering::Relaxed) {
            self.metrics.miss.fetch_add(1, Ordering::Relaxed);
            return false;
        }

        let hashed = self.hasher.hash_one(key);
        let exists = self.filter.read().contains(&hashed);

        if exists {
            self.metrics.hit.fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics.miss.fetch_add(1, Ordering::Relaxed);
        }

        exists
    }

    /// Replaces the internal [`BinaryFuse32`] by consuming a list of items.
    ///
    /// NOTE: `BinaryFuse32` construction requires all keys to be distinct. This method does *not*
    /// de-duplicate keys; the caller should enforce uniqueness (e.g. `SELECT DISTINCT` at the DB
    /// layer).
    pub async fn replace_from<K>(&self, keys: &[K]) -> Result<u64, Error>
    where
        K: std::hash::Hash,
    {
        let cap = u64::try_from(keys.len()).unwrap_or(u64::MAX);

        if keys.is_empty() {
            self.is_empty.store(true, Ordering::Relaxed);
            self.metrics.cap.store(cap, Ordering::Relaxed);
            return Ok(cap);
        }

        let filter =
            BinaryFuse32::try_from(keys.iter().map(|it| self.hasher.hash_one(it)).collect_vec())
                .map_err(|e| Error::with_code(StatusCode::INTERNAL_SERVER_ERROR, e))?;

        *self.filter.write() = filter;
        self.is_empty.store(false, Ordering::Relaxed);
        self.metrics.cap.store(cap, Ordering::Relaxed);
        Ok(cap)
    }

    pub fn mark(&self) {
        self.dirt.swap(true, Ordering::Relaxed);
    }
}

impl Default for FilterWithMetrics<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ManagedFilters<'a> {
    application_id_filter: FilterWithMetrics<'a>,
    secret_id_filter: FilterWithMetrics<'a>,
    secret_filter: FilterWithMetrics<'a>,
}

fn spawn_refresh_worker<K, F, Fut>(
    filter: FilterWithMetrics<'static>,
    database: DatabaseConnection,
    label: &'static str,
    fetch_desc: &'static str,
    fetch: F,
) where
    K: std::hash::Hash + Send + Sync + 'static,
    F: Fn(DatabaseConnection) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<K>, Error>> + Send + 'static,
{
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(50)).await;

            if !filter.dirt.load(Ordering::Relaxed) {
                continue;
            }

            match fetch(database.clone()).await {
                Ok(keys) => match filter.replace_from(&keys).await {
                    Ok(cap) => {
                        filter.dirt.store(false, Ordering::Relaxed);
                        debug!("refreshed {label} with {cap} keys");
                    }
                    Err(error) => {
                        debug!("failed to refresh {label}: {error}");
                    }
                },
                Err(error) => {
                    debug!("failed to fetch {fetch_desc} for {label}: {error}");
                }
            }
        }
    });
}

impl ManagedFilters<'_> {
    pub fn new<'a: 'static>(database: DatabaseConnection) -> ManagedFilters<'a> {
        let instance = ManagedFilters {
            application_id_filter: FilterWithMetrics::new(),
            secret_id_filter: FilterWithMetrics::new(),
            secret_filter: FilterWithMetrics::new(),
        };

        spawn_refresh_worker(
            instance.application_id_filter.clone(),
            database.clone(),
            "application_id_filter",
            "application ids",
            |database| async move { Applications::get_all_application_ids(&database).await },
        );

        spawn_refresh_worker(
            instance.secret_id_filter.clone(),
            database.clone(),
            "secret_id_filter",
            "secret ids",
            |database| async move { ApplicationSecrets::get_all_secret_ids(&database).await },
        );

        spawn_refresh_worker(
            instance.secret_filter.clone(),
            database,
            "secret_filter",
            "secrets",
            |database| async move { ApplicationSecrets::get_all_secrets(&database).await },
        );

        instance
    }
}

impl<'a> ManagedFilters<'a> {
    pub fn application_id_filter(&'_ self) -> &FilterWithMetrics<'_> {
        &self.application_id_filter
    }

    pub fn secret_id_filter(&self) -> &FilterWithMetrics<'_> {
        &self.secret_id_filter
    }

    pub fn secret_filter(&self) -> &FilterWithMetrics<'_> {
        &self.secret_filter
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::FilterWithMetrics;

    // NOTE: AI-generated test
    #[test]
    fn exists_updates_hit_and_miss_metrics() {
        let filter = FilterWithMetrics::new();
        let keys = [1_u64, 2, 3, 4];

        futures::executor::block_on(filter.replace_from(&keys))
            .expect("replace_from should succeed");

        let hit0 = filter.metrics.hit.load(Ordering::Relaxed);
        let miss0 = filter.metrics.miss.load(Ordering::Relaxed);

        assert!(filter.exists(&1_u64));
        assert_eq!(filter.metrics.hit.load(Ordering::Relaxed), hit0 + 1);
        assert_eq!(filter.metrics.miss.load(Ordering::Relaxed), miss0);

        let mut found_miss = false;
        for k in 1_000_000_u64..1_010_000_u64 {
            let hit_before = filter.metrics.hit.load(Ordering::Relaxed);
            let miss_before = filter.metrics.miss.load(Ordering::Relaxed);

            let exists = filter.exists(&k);

            let hit_after = filter.metrics.hit.load(Ordering::Relaxed);
            let miss_after = filter.metrics.miss.load(Ordering::Relaxed);

            if exists {
                assert_eq!(hit_after, hit_before + 1);
                assert_eq!(miss_after, miss_before);
            } else {
                assert_eq!(hit_after, hit_before);
                assert_eq!(miss_after, miss_before + 1);
                found_miss = true;
                break;
            }
        }

        assert!(
            found_miss,
            "expected to observe at least one miss (got all false positives)"
        );
    }

    #[test]
    fn empty_filter_always_misses() {
        let filter = FilterWithMetrics::new();

        futures::executor::block_on(filter.replace_from::<u64>(&[]))
            .expect("replace_from should support empty key sets");

        assert!(!filter.exists(&42_u64));
        assert_eq!(filter.metrics.cap.load(Ordering::Relaxed), 0);
        assert_eq!(filter.metrics.hit.load(Ordering::Relaxed), 0);
        assert_eq!(filter.metrics.miss.load(Ordering::Relaxed), 1);
    }
}
