use mimalloc::MiMalloc;
use oceaniam::error::Error;
use oceaniam_common::config::BackendConfig;
use oceaniam_database::setup::{connect, init_system};
use oceaniam_worker::runtime::{WorkerContext, WorkerRuntime};
use tracing::error;
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

    let database = connect(&config.database)
        .await
        .inspect_err(|e| error!(error = %e, "failed to connect worker database"))?;

    init_system(&database)
        .await
        .inspect_err(|e| error!(error = %e, "failed to init system data"))?;

    WorkerRuntime::new(WorkerContext { database })?.run().await
}
