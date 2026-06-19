//! Secret management-related API endpoints

use crate::{conversion::secrets::with_application_ids, error::AppResult};
use axum::extract::{Path, State};
use axum_extra::extract::OptionalQuery;
use oceaniam_api::{ApiResponse, Empty, ErrorResponse, PageParam, PagedResponse};
use oceaniam_audit::types::{
    AuditPayload, CreateApplicationSecretPayload, DeleteApplicationSecretPayload,
};
use oceaniam_common::sqid::Sqid;
use oceaniam_vo::applications::SecretVO;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    middlewares::permission::{PlatformPermissionGuard, SecretCreate, SecretDelete, SecretRead},
    state::AppState,
};

#[utoipa::path(
        post,
        path = "/secrets",
        tag = "Secrets",
        params(("Authorization" = String, Header, description = "Bearer token")),
        responses(
            (status = 200, body = ApiResponse<SecretVO>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "secrets.create",
    skip(auth, applications, auditing),
    fields(secret_id = field::Empty)
)]
pub async fn create_secret(
    auth: PlatformPermissionGuard<SecretCreate>,
    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState>,
) -> AppResult<SecretVO> {
    let operator_id = auth.claim.sub;
    let model = applications
        .secrets()
        .create_secret()
        .await
        .inspect_err(|e| {
            error!(error = %e, "failed to create secret");
        })?;
    Span::current().tap(|it| {
        it.record("secret_id", field::display(&model.id));
    });

    info!(secret_id = %model.id, "secret created successfully");

    auditing
        .write(AuditPayload::from(CreateApplicationSecretPayload {
            operator_id,
            secret_id: model.id,
        }))
        .await;

    Ok(ApiResponse::new(
        crate::conversion::secrets::secret_with_unmasked(model),
    ))
}

#[utoipa::path(
        get,
        path = "/secrets",
        tag = "Secrets",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("page" = Option<u64>, Query, description = "Page number"),
            ("per_page" = Option<u64>, Query, description = "Items per page"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<SecretVO>>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "secrets.list",
    skip(applications),
    fields(page = field::Empty, per_page = field::Empty)
)]
pub async fn get_secrets(
    _: PlatformPermissionGuard<SecretRead>,
    OptionalQuery(query): OptionalQuery<PageParam>,
    State(AppState { applications, .. }): State<AppState>,
) -> AppResult<PagedResponse<SecretVO>> {
    let page: PageParam = query.unwrap_or_default().into_clamped();

    Span::current().tap(|it| {
        it.record("page", page.page)
            .record("per_page", page.per_page);
    });

    let PagedResponse {
        items: secrets,
        page_info,
    } = applications
        .secrets()
        .get_secret_models(page)
        .await
        .inspect_err(|e| {
            error!(error = %e, "failed to fetch secrets");
        })?;

    let secret_ids = secrets.iter().map(|secret| secret.id).collect();

    let application_ids_by_secret = applications
        .secrets()
        .get_secret_application_ids_batch_by_ids(secret_ids)
        .await
        .inspect_err(|e| {
            error!(error = %e, "failed to fetch secret bindings in batch");
        })?;
    let items: Vec<_> = secrets
        .into_par_iter()
        .map(|secret| {
            let application_ids = application_ids_by_secret
                .get(&secret.id)
                .cloned()
                .unwrap_or_default();
            with_application_ids(
                crate::conversion::secrets::secret_with_masked(secret),
                application_ids,
            )
        })
        .collect();

    Ok(ApiResponse::new(PagedResponse { items, page_info }))
}

#[utoipa::path(
        get,
        path = "/secrets/{secret_id}",
        tag = "Secrets",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("secret_id" = String, Path, description = "Secret ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<SecretVO>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Secret not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "secrets.get",
    skip(applications, secret_id),
    fields(secret_id = field::Empty)
)]
pub async fn get_secret(
    _: PlatformPermissionGuard<SecretRead>,
    Path(secret_id): Path<Sqid>,
    State(AppState { applications, .. }): State<AppState>,
) -> AppResult<SecretVO> {
    let secret_id: Uuid = secret_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert secret_id"))?;
    Span::current().tap(|it| {
        it.record("secret_id", field::display(&secret_id));
    });

    let secret = applications.secrets().get_secret(secret_id).await?;
    let application_ids = applications
        .secrets()
        .get_secret_application_ids(secret_id)
        .await?;

    Ok(ApiResponse::new(with_application_ids(
        crate::conversion::secrets::secret_with_masked(secret),
        application_ids,
    )))
}

#[utoipa::path(
        delete,
        path = "/secrets/{secret_id}",
        tag = "Secrets",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("secret_id" = String, Path, description = "Secret ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Secret not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "secrets.delete",
    skip(auth, applications, auditing, secret_id),
    fields(secret_id = field::Empty)
)]
pub async fn delete_secret(
    auth: PlatformPermissionGuard<SecretDelete>,
    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState>,
    Path(secret_id): Path<Sqid>,
) -> AppResult<Empty> {
    let operator_id = auth.claim.sub;
    let secret_id: Uuid = secret_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert secret_id"))?;
    Span::current().tap(|it| {
        it.record("secret_id", field::display(&secret_id));
    });

    applications
        .secrets()
        .delete_secret_by_id(secret_id)
        .await
        .inspect_err(|e| {
            error!(secret_id = %secret_id, error = %e, "failed to delete secret");
        })?;

    info!(secret_id = %secret_id, "secret deleted successfully");

    auditing
        .write(AuditPayload::from(DeleteApplicationSecretPayload {
            operator_id,
            secret_id,
        }))
        .await;

    Ok(ApiResponse::new(Empty::default()))
}

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(create_secret))
        .routes(routes!(get_secrets))
        .routes(routes!(get_secret))
        .routes(routes!(delete_secret))
}
