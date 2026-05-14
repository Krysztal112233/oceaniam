use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use mimalloc::MiMalloc;
use oceaniam::app::{app, build_openapi_spec, build_state};
use oceaniam::error::Error;
use oceaniam_common::config::BackendConfig;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser)]
#[command(name = "oceaniam")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate OpenAPI specification
    Openapi {
        /// Output directory for the generated OpenAPI spec
        #[arg(long, default_value = ".")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Openapi { output }) => generate_openapi(&output).await,
        None => run_server().await,
    }
}

async fn run_server() -> Result<(), Error> {
    let _ = dotenvy::dotenv();

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_ansi(false)
        .init();

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

async fn generate_openapi(output: impl AsRef<Path>) -> Result<(), Error> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let openapi = build_openapi_spec();

    let path_count = openapi.paths.paths.len();
    let schema_count = openapi
        .components
        .as_ref()
        .map(|c| c.schemas.len())
        .unwrap_or(0);

    info!("OpenAPI Spec Paths:      {path_count:<29}");
    info!("OpenAPI Spec Schemas:    {schema_count:<29}");

    let content = serde_json::to_string_pretty(&openapi)?;

    let _ = fs::write(output, content)?;

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
