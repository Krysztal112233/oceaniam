use axum::Router;
use log::{debug, error};
use mimalloc::MiMalloc;
use oceaniam_common::config::{BackendConfig, DatabaseConfig};
use oceaniam_common::error::Error;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tap::Pipe;
use tower_http::trace::TraceLayer;
use utoipa::openapi::Contact;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::state::AppState;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod endpoints;
mod keybox;
mod middlewares;
#[allow(unused)]
mod roller;
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

    Ok(db)
}
