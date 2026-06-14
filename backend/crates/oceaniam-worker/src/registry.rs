use std::collections::HashMap;
use std::sync::Arc;

use linkme::distributed_slice;
use oceaniam_common::crypto::MasterKey;
use oceaniam_worker_runtime::Worker;
use sea_orm::DatabaseConnection;

use crate::error::Error;

#[derive(Clone)]
pub struct WorkerContext {
    pub database: DatabaseConnection,
    pub master_key: Arc<MasterKey>,
}

pub type OceaniamWorker = dyn Worker<WorkerContext, Error = Error>;
pub type OceaniamWorkerRef = std::sync::Arc<OceaniamWorker>;
pub type OceaniamWorkerFactory = fn() -> OceaniamWorkerRef;

#[distributed_slice]
pub static REGISTERED_WORKERS: [OceaniamWorkerFactory];

pub fn collect_workers() -> HashMap<String, OceaniamWorkerRef> {
    REGISTERED_WORKERS
        .iter()
        .map(|factory| {
            let worker = factory();
            let name = worker.name().to_owned();
            (name, worker)
        })
        .collect()
}
