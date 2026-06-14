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

    let master_key =
        oceaniam_common::crypto::MasterKey::from_hex(&config.master_key).map_err(|e| {
            tracing::error!(error = %e, "failed to parse `OCEANIAM_MASTER_KEY`");
            Error::Internal {
                msg: format!("invalid master key: {e}"),
                location: snafu::location!(),
            }
        })?;

    let state = AppState::new(
        database.clone(),
        config.clone(),
        std::sync::Arc::new(master_key.clone()),
    )
    .await?;

    health_check_kek(&database, &master_key).await?;

    Ok(state)
}

/// Verify the KEK is correct by decrypting one existing key (if any).
/// Fresh installs (empty `key_boxes`) skip this check.
async fn health_check_kek(
    database: &sea_orm::DatabaseConnection,
    master_key: &oceaniam_common::crypto::MasterKey,
) -> Result<(), Error> {
    use oceaniam_database::model::key_boxes::Entity as KeyBoxes;
    use sea_orm::{EntityTrait, PaginatorTrait};

    let count = KeyBoxes::find().count(database).await?;

    if count == 0 {
        tracing::info!("`key_boxes` empty; skipping KEK health check");
        return Ok(());
    }

    let row = KeyBoxes::find()
        .one(database)
        .await?
        .ok_or_else(|| Error::Internal {
            msg: "`key_boxes` COUNT > 0 but no row returned".to_string(),
            location: snafu::location!(),
        })?;

    use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
    let field: oceaniam_keybox::SecretField =
        serde_json::from_value(row.secret).map_err(|e| Error::Internal {
            msg: format!("failed to deserialize `SecretField`: {e}"),
            location: snafu::location!(),
        })?;

    let blob = oceaniam_common::crypto::EncryptedBlob {
        nonce: B64
            .decode(&field.nonce)
            .map_err(|e| Error::Internal {
                msg: format!("nonce base64 decode: {e}"),
                location: snafu::location!(),
            })?
            .try_into()
            .map_err(|_| Error::Internal {
                msg: "nonce must be 24 bytes".to_string(),
                location: snafu::location!(),
            })?,
        ciphertext: B64.decode(&field.ciphertext).map_err(|e| Error::Internal {
            msg: format!("ciphertext base64 decode: {e}"),
            location: snafu::location!(),
        })?,
        key_version: field.key_version,
    };

    match master_key.decrypt(&blob) {
        Ok(_) => {
            tracing::info!("KEK health check passed");
            Ok(())
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                "KEK health check FAILED — `OCEANIAM_MASTER_KEY` does not match the key used during migration"
            );
            Err(Error::Internal {
                msg: "KEK mismatch: `OCEANIAM_MASTER_KEY` does not match encrypted data. \
                      Verify it matches the key used during migration."
                    .to_string(),
                location: snafu::location!(),
            })
        }
    }
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
