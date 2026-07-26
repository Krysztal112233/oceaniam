use std::sync::Arc;

use mimalloc::MiMalloc;
use oceaniam_common::config::BackendConfig;
use oceaniam_database::setup::{connect, init_system};
use oceaniam_worker::error::Error;
use oceaniam_worker::{WorkerContext, collect_workers};
use oceaniam_worker_runtime::WorkerRuntime;
use tokio::signal::unix::{SignalKind, signal};
use tracing::{debug, error};
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_ansi(false)
        .init();

    let _ = dotenvy::dotenv();

    let config =
        BackendConfig::new().inspect_err(|e| error!(error = %e, "failed to load worker config"))?;

    let BackendConfig {
        database: database_config,
        master_key: master_key_hex,
        ..
    } = config;
    let master_key = Arc::new(
        oceaniam_common::crypto::MasterKey::from_hex_owned(master_key_hex).map_err(|e| {
            error!(error = %e, "failed to parse `OCEANIAM_MASTER_KEY`");
            Error::Internal {
                msg: format!("invalid master key: {e}"),
                location: snafu::location!(),
            }
        })?,
    );

    let database = connect(&database_config)
        .await
        .inspect_err(|e| error!(error = %e, "failed to connect worker database"))?;

    init_system(&database)
        .await
        .inspect_err(|e| error!(error = %e, "failed to init system data"))?;

    let workers = collect_workers();
    let ctrl = WorkerRuntime::new(
        WorkerContext {
            database,
            master_key,
        },
        workers,
    )
    .start()?;

    let mut terminate =
        signal(SignalKind::terminate()).expect("failed to install SIGTERM signal handler");
    let mut interrupt =
        signal(SignalKind::interrupt()).expect("failed to install SIGINT signal handler");

    tokio::select! {
        _ = terminate.recv() => debug!("received SIGTERM, starting worker shutdown"),
        _ = interrupt.recv() => debug!("received SIGINT, starting worker shutdown"),
    }

    ctrl.shutdown().await
}
