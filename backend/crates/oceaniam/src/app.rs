use crate::error::Error;
use axum::{
    Router,
    http::{HeaderValue, header},
};
use oceaniam_common::config::{BackendConfig, CorsConfig};
use tap::Pipe;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::openapi::{Components, Contact, ObjectBuilder, Type};
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::{endpoints, state::AppState};

/// Build the OpenAPI spec without requiring a database connection.
pub fn build_openapi_spec() -> utoipa::openapi::OpenApi {
    let (_, mut openapi) = OpenApiRouter::<AppState>::new()
        .pipe(endpoints::endpoint)
        .split_for_parts();

    // NOTE: utoipa generates `$ref` for enum types used as query params but omits the definition
    // from `components/schemas`, causing openapi-generator-cli to abort.
    let components = openapi.components.get_or_insert_with(Components::new);
    components
        .schemas
        .entry("ApplicationUsersSortOrder".to_string())
        .or_insert_with(|| {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .enum_values(Some(["asc", "desc"]))
                .into()
        });

    openapi.info.title = "OceanIAM".to_string();
    openapi.info.description = Some("Pretty simple IAM implemented in Rust".to_string());
    openapi.info.contact = Some(
        utoipa::openapi::Contact::builder()
            .email(Some("krysztal.huang@outlook.com"))
            .name(Some("Krysztal Huang"))
            .build(),
    );

    openapi
}

pub async fn build_state(config: &BackendConfig) -> Result<AppState, Error> {
    let database = crate::setup_database(&config.database).await?;
    AppState::new(database, config.clone()).await
}

pub fn app(state: AppState, cors: CorsConfig) -> Router {
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

    router
        .merge(Scalar::with_url("/docs", openapi))
        .layer(to_cors_layer(cors))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn to_cors_layer(CorsConfig { allow_origin }: CorsConfig) -> CorsLayer {
    CorsLayer::new()
        .allow_headers([
            header::ACCEPT,
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("x-oceaniam-token-dispatch"),
        ])
        .allow_methods(Any)
        .allow_origin(allow_origin.parse::<HeaderValue>().unwrap())
}
