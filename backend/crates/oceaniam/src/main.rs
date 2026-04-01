use std::time::Duration;

use mimalloc::MiMalloc;
use oceaniam_common::{
    config::{BackendConfig, DatabaseConfig},
    consts,
    error::Error,
};
use oceaniam_database::{
    helper::{applications::ApplicationHelper, tenants::TenantsHelper},
    model::prelude::*,
};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tap::Pipe;
use tracing::{debug, error, warn};
use tracing_subscriber::EnvFilter;

use crate::app::{app, build_state};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod app;
mod endpoints;
mod middlewares;
mod state;

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

async fn setup_database(
    DatabaseConfig {
        dsn,
        slow_statements_logging_threshold,
        max_connections,
        min_connections,
    }: &DatabaseConfig,
) -> Result<DatabaseConnection, Error> {
    let options = ConnectOptions::new(dsn)
        .pipe_borrow_mut(|it| match slow_statements_logging_threshold {
            Some(milis) => it.sqlx_slow_statements_logging_settings(
                log::LevelFilter::Warn,
                Duration::from_micros(*milis),
            ),
            _ => it,
        })
        .pipe_borrow_mut(|it| match max_connections {
            Some(c) => it.max_connections(*c),
            _ => it,
        })
        .pipe_borrow_mut(|it| match min_connections {
            Some(c) => it.min_connections(*c),
            _ => it,
        })
        .pipe_borrow_mut(|it| it.sqlx_logging(false))
        .to_owned();

    let db = Database::connect(options).await.inspect_err(|err| {
        error!(
            %dsn,
            error = %err,
            "failed to connect to database"
        )
    })?;

    init_system(&db).await?;

    Ok(db)
}

async fn init_system(db: &DatabaseConnection) -> Result<(), Error> {
    if !Tenants::is_system_tenant_exist(db).await? {
        warn!(
            tenant_id = %consts::SYSTEM_TENANT_UUID,
            "system tenant missing; creating builtin tenant"
        );

        Tenants::create_tenant(
            consts::SYSTEM_TENANT_UUID,
            Some("System builtin tenant"),
            db,
        )
        .await?;
    }

    if !Applications::is_system_application_exist(db).await? {
        warn!(
            tenant_id = %consts::SYSTEM_TENANT_UUID,
            application_id = %consts::SYSTEM_APPLICATION_UUID,
            "system application missing; creating builtin application"
        );
        Applications::create_application(
            consts::SYSTEM_APPLICATION_UUID,
            consts::SYSTEM_TENANT_UUID,
            db,
        )
        .await?;
    }

    Ok(())
}
