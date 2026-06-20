use axum::{
    extract::{FromRequestParts, Path},
    http::StatusCode,
    http::request::Parts,
};
use oceaniam_common::sqid::Sqid;
use oceaniam_database::model;
use tap::{Pipe, Tap};
use tracing::{Span, error, field};
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

use crate::{error::Error, state::AppState};

mod apps;
mod challenges;
mod configuration;
mod keys;
mod roles;
mod secrets;
mod statistics;
mod tokens;
mod users;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct TenantApplicationPath {
    pub tenant_id: Sqid,
    pub application_id: Sqid,
}

#[derive(Debug, Clone)]
pub struct ResolvedApplication(pub model::applications::Model);

impl std::ops::Deref for ResolvedApplication {
    type Target = model::applications::Model;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ResolvedApplication {
    pub fn id(&self) -> uuid::Uuid {
        self.0.id
    }

    pub fn tenant_id(&self) -> uuid::Uuid {
        self.0.tenant_id
    }
}

impl FromRequestParts<AppState> for ResolvedApplication {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Path(path) = Path::<TenantApplicationPath>::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::with_code(StatusCode::BAD_REQUEST, "invalid path parameters"))?;

        let tenant_id: Uuid = path.tenant_id.try_into()?;
        let application_id: Uuid = path
            .application_id
            .try_into()
            .inspect_err(|e| error!(error = %e, "failed to convert application_id"))?;
        Span::current().tap(|it| {
            it.record("tenant_id", field::display(&tenant_id))
                .record("application_id", field::display(&application_id));
        });

        let app = state
            .applications
            .get_model(application_id)
            .await
            .inspect_err(|e| {
                error!(
                    %tenant_id,
                    %application_id,
                    error = %e,
                    "failed to get application detail"
                )
            })?;

        if app.tenant_id != tenant_id {
            return Err(Error::with_code(
                StatusCode::NOT_FOUND,
                format!("application_id={application_id} not found under tenant_id={tenant_id}",),
            ));
        }

        Ok(ResolvedApplication(app.as_ref().clone()))
    }
}

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .pipe(apps::endpoint)
        .pipe(challenges::endpoint)
        .pipe(configuration::endpoint)
        .pipe(keys::endpoint)
        .pipe(roles::endpoint)
        .pipe(secrets::endpoint)
        .pipe(tokens::endpoint)
        .pipe(users::endpoint)
        .pipe(statistics::endpoint)
}
