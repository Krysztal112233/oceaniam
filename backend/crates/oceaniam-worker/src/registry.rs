use std::collections::HashMap;
use std::sync::Arc;

use linkme::distributed_slice;
use oceaniam_common::crypto::MasterKey;
use oceaniam_worker_runtime::WorkerRegistration;
use sea_orm::DatabaseConnection;

use crate::error::Error;

#[derive(Clone)]
pub struct WorkerContext {
    pub database: DatabaseConnection,
    pub master_key: Arc<MasterKey>,
}

pub type OceaniamWorkerRegistration = WorkerRegistration<WorkerContext, Error>;
pub type OceaniamWorkerFactory = fn() -> OceaniamWorkerRegistration;

#[distributed_slice]
pub static REGISTERED_WORKERS: [OceaniamWorkerFactory];

pub fn collect_workers() -> HashMap<String, OceaniamWorkerRegistration> {
    REGISTERED_WORKERS
        .iter()
        .map(|factory| {
            let registration = factory();
            let name = registration.worker.name().to_owned();
            (name, registration)
        })
        .collect()
}
