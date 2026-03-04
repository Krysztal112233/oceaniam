use std::{
    hash::BuildHasher,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use axum::http::StatusCode;
use itertools::Itertools;
use oceaniam_common::error::Error;
use parking_lot::RwLock;
use rapidhash::fast::SeedableState;
use xorf::BinaryFuse32;
use xorf::Filter as _;

#[derive(Debug, Default)]
struct Metrics {
    hit: AtomicU64,
    miss: AtomicU64,
    cap: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct Filter<'a> {
    hasher: SeedableState<'a>,
    filter: Arc<RwLock<BinaryFuse32>>,
    metrics: Arc<Metrics>,
}

impl Filter<'_> {
    pub fn new() -> Self {
        let metrics = Arc::new(Metrics::default());
        metrics.cap.store(1, Ordering::Relaxed);

        Self {
            hasher: SeedableState::random(),
            filter: Arc::new(RwLock::new(
                BinaryFuse32::try_from_iterator(vec![10].into_iter()).unwrap(),
            )),
            metrics,
        }
    }

    pub fn exists<K>(&self, key: &K) -> bool
    where
        K: std::hash::Hash,
    {
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
        let filter =
            BinaryFuse32::try_from(keys.iter().map(|it| self.hasher.hash_one(it)).collect_vec())
                .map_err(|e| Error::with_code(StatusCode::INTERNAL_SERVER_ERROR, e))?;

        *self.filter.write() = filter;

        let cap = u64::try_from(keys.len()).unwrap_or(u64::MAX);
        self.metrics.cap.store(
            u64::try_from(keys.len()).unwrap_or(u64::MAX),
            Ordering::Relaxed,
        );
        Ok(cap)
    }
}

impl Default for Filter<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::Filter;

    // NOTE: AI-generated test
    #[test]
    fn exists_updates_hit_and_miss_metrics() {
        let filter = Filter::new();
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
}
