use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use mimalloc::MiMalloc;
use oceaniam::app::{app, build_openapi_spec, build_state};
use oceaniam::error::Error;
use oceaniam_common::config::BackendConfig;
use oceaniam_telemetry::{ProcessKind, init as init_telemetry};
use tracing::{debug, error, info};

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
#[allow(clippy::result_large_err)]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Openapi { output }) => generate_openapi(&output).await,
        None => run_server().await,
    }
}

async fn run_server() -> Result<(), Error> {
    let _ = dotenvy::dotenv();

    let config = BackendConfig::new().map_err(|error| {
        eprintln!("failed to load backend config: {error}");
        Error::Internal {
            msg: error.to_string(),
            location: snafu::location!(),
        }
    })?;
    let _telemetry =
        init_telemetry(&config.telemetry, ProcessKind::Server, "debug").map_err(|error| {
            Error::Internal {
                msg: error.to_string(),
                location: snafu::location!(),
            }
        })?;

    let addr = config.addr.clone();
    let cors = config.cors.clone();
    let states = build_state(config)
        .await
        .inspect_err(|e| error!(error = %e, "failed to build application state"))?;

    // Spawn the development-account expiration consumer. It polls the pgmq queue whose delayed
    // messages act as the dev-account timer; it must run inside the API process because
    // deletion performs moka cache eviction (see `state::dev_account_expiry`). The consumer is
    // supervised: a panic is logged and the consumer respawned so dev accounts keep being
    // deleted (lazy rejection at sign-in/refresh bounds the damage while it is down).
    {
        let consumer_state = states.clone();
        tokio::spawn(async move {
            loop {
                let result = tokio::spawn(oceaniam::state::dev_account_expiry::run(
                    consumer_state.clone(),
                    shutdown_signal(),
                ))
                .await;
                match result {
                    // Graceful shutdown requested.
                    Ok(()) => break,
                    Err(join_error) => {
                        error!(error = %join_error, "dev account expiration consumer died; respawning");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });
    }

    let router = app(states, cors);

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
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"));

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

    fs::write(output, content)?;

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
