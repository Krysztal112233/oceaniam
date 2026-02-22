use axum::Router;
use log::{debug, error, warn};
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
use utoipa::openapi::Contact;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::state::AppState;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod credentials;
mod endpoints;
mod keybox;
mod middlewares;
mod revoked;
#[allow(unused)]
mod roller;
mod secrets;
mod state;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config = BackendConfig::new().inspect_err(|e| error!("{e}"))?;

    let states = {
        let database = setup_database(&config.database)
            .await
            .inspect_err(|e| error!("{e}"))?;

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

    let listener = tokio::net::TcpListener::bind(config.addr)
        .await
        .inspect_err(|e| error!("{e}"))?;

    axum::serve(listener, router).await?;

    Ok(())
}

async fn setup_database(config: &DatabaseConfig) -> Result<DatabaseConnection, Error> {
    let options = ConnectOptions::from(config.clone());

    let db = Database::connect(options)
        .await
        .inspect_err(|err| error!("{err}"))?;

    debug!("connected to database: {}", config.dsn);

    if !Tenants::is_exist(consts::SYSTEM_TENANT_UUID, &db).await? {
        warn!("the system tenant does not exist; a system tenant is about to be created.");

        Tenants::create_tenant(
            consts::SYSTEM_TENANT_UUID,
            Some("System builtin tenant"),
            &db,
        )
        .await?;
    }

    if !Applications::is_exist(consts::SYSTEM_APPLICATION_UUID, &db).await? {
        warn!("system application does not exist; creating system application now.");
        Applications::create_application(
            consts::SYSTEM_APPLICATION_UUID,
            consts::SYSTEM_TENANT_UUID,
            &db,
        )
        .await?;
    }

    Ok(db)
}
