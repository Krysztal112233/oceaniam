use mimalloc::MiMalloc;
use oceaniam::app::{app, build_state};
use oceaniam_common::{config::BackendConfig, error::Error};
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

    let config = BackendConfig::new()
        .inspect_err(|e| error!(error = %e, "failed to load backend config"))?;

    let states = build_state(&config)
        .await
        .inspect_err(|e| error!(error = %e, "failed to build application state"))?;

    let router = app(states, config.cors.clone());

    let addr = config.addr.clone();
    let listener = tokio::net::TcpListener::bind(addr.clone())
        .await
        .inspect_err(|e| error!(addr = %addr, error = %e, "failed to bind tcp listener"))?;

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    debug!("http server stopped");

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate =
        signal(SignalKind::terminate()).expect("failed to install SIGTERM signal handler");
    let mut interrupt =
        signal(SignalKind::interrupt()).expect("failed to install SIGINT signal handler");

    tokio::select! {
        _ = terminate.recv() => debug!("received SIGTERM, starting graceful shutdown"),
        _ = interrupt.recv() => debug!("received SIGINT, starting graceful shutdown"),
    }
}
