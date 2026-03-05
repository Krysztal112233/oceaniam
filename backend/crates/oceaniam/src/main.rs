use axum::Router;
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
use tower_http::trace::TraceLayer;
use tracing::{debug, error, warn};
use tracing_subscriber::EnvFilter;
use utoipa::openapi::Contact;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::state::AppState;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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

    let states = {
        let database = setup_database(&config.database)
            .await
            .inspect_err(|e| error!(error = %e, "failed to setup database"))?;

        AppState::new(database).await?
    };

    let (router, mut openapi) = OpenApiRouter::new()
        .pipe(endpoints::endpoint)
        .split_for_parts();

    {
        openapi.info.title = "OceanIAM".to_string();
        openapi.info.description = Some("Pretty simple IAM implemented in Rust".to_string());
        openapi.info.contact = Some(
            Contact::builder()
                .email(Some("krysztal.huang@outlook.com"))
                .name(Some("Krysztal Huang"))
                .build(),
        );
    }

    let router: Router = router
        .merge(Scalar::with_url("/docs", openapi))
        .layer(TraceLayer::new_for_http())
        .with_state(states);

    let addr = config.addr.clone();
    let listener = tokio::net::TcpListener::bind(addr.clone())
        .await
        .inspect_err(|e| error!(addr = %addr, error = %e, "failed to bind tcp listener"))?;

    axum::serve(listener, router).await?;

    Ok(())
}

async fn setup_database(config: &DatabaseConfig) -> Result<DatabaseConnection, Error> {
    let options = ConnectOptions::from(config.clone());

    let db = Database::connect(options).await.inspect_err(|err| {
        error!(
            dsn = %redact_dsn(&config.dsn),
            error = %err,
            "failed to connect to database"
        )
    })?;

    debug!(dsn = %redact_dsn(&config.dsn), "connected to database");

    if !Tenants::is_exist(consts::SYSTEM_TENANT_UUID, &db).await? {
        warn!(
            tenant_id = %consts::SYSTEM_TENANT_UUID,
            "system tenant missing; creating builtin tenant"
        );

        Tenants::create_tenant(
            consts::SYSTEM_TENANT_UUID,
            Some("System builtin tenant"),
            &db,
        )
        .await?;
    }

    if !Applications::is_exist(consts::SYSTEM_APPLICATION_UUID, &db).await? {
        warn!(
            tenant_id = %consts::SYSTEM_TENANT_UUID,
            application_id = %consts::SYSTEM_APPLICATION_UUID,
            "system application missing; creating builtin application"
        );
        Applications::create_application(
            consts::SYSTEM_APPLICATION_UUID,
            consts::SYSTEM_TENANT_UUID,
            &db,
        )
        .await?;
    }

    Ok(db)
}

fn redact_dsn(dsn: &str) -> String {
    let mut out = dsn.to_string();

    // Redact URL userinfo passwords (e.g. `postgres://user:pass@host/db`).
    if let Some(scheme_end) = out.find("://") {
        let userinfo_start = scheme_end + 3;
        if let Some(at_offset) = out[userinfo_start..].find('@') {
            let at_pos = userinfo_start + at_offset;
            if let Some(colon_offset) = out[userinfo_start..at_pos].find(':') {
                let colon_pos = userinfo_start + colon_offset;
                out.replace_range(colon_pos + 1..at_pos, "***");
            }
        }
    }

    // Redact `password=` key/value segments (e.g. `... password=pass ...` or `...?password=pass&...`).
    let lower = out.to_ascii_lowercase();
    let key = "password=";

    let mut redacted = String::with_capacity(out.len());
    let mut start = 0usize;

    while let Some(pos) = lower[start..].find(key) {
        let abs_pos = start + pos;
        let key_end = abs_pos + key.len();

        redacted.push_str(&out[start..key_end]);

        let mut value_end = key_end;
        while value_end < out.len() {
            match out.as_bytes()[value_end] {
                b'&' | b' ' | b';' | b'\t' | b'\n' | b'\r' | b'#' => break,
                _ => value_end += 1,
            }
        }

        redacted.push_str("***");
        start = value_end;
    }

    redacted.push_str(&out[start..]);
    redacted
}
