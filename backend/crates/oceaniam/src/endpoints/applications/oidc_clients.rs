use std::collections::{HashMap, HashSet};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::extract::OptionalQuery;
use axum_valid::Garde;
use oceaniam_api::{ApiResponse, ErrorResponse, PageParam, PagedResponse};
use oceaniam_audit::types::{AuditPayload, CreateOidcClientPayload};
use oceaniam_common::{consts, sqid::Sqid};
use oceaniam_database::{helper::oidc_clients::OidcClientsHelper, model::prelude::OidcClients};
use oceaniam_oidc::{RedirectUriPolicy, validate_redirect_uri};
use oceaniam_vo::oidc_clients::{CreateOidcClientRequest, OidcClientVO};
use sea_orm::TransactionTrait;
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::ResolvedApplication;
use crate::{
    error::{AppResult, Error},
    middlewares::permission::{OidcClientCreate, OidcClientRead, PlatformPermissionGuard},
    state::AppState,
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(list_oidc_clients))
        .routes(routes!(create_oidc_client))
        .routes(routes!(get_oidc_client))
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct OidcClientsListQuery {
    page: Option<u64>,
    per_page: Option<u64>,
}

impl OidcClientsListQuery {
    fn into_page(self) -> Result<PageParam, Error> {
        let defaults = PageParam::default();
        let page = PageParam {
            page: self.page.unwrap_or(defaults.page),
            per_page: self.per_page.unwrap_or(defaults.per_page),
        }
        .into_clamped();
        let offset = page
            .page
            .saturating_sub(1)
            .checked_mul(page.per_page)
            .filter(|offset| *offset <= i64::MAX as u64);

        if offset.is_none() {
            return Err(Error::with_code(
                StatusCode::BAD_REQUEST,
                "pagination offset is too large",
            ));
        }

        Ok(page)
    }
}

fn ensure_oidc_management_application(app: &ResolvedApplication) -> Result<(), Error> {
    if app.tenant_id() == consts::SYSTEM_TENANT_UUID {
        return Err(Error::with_code(
            StatusCode::NOT_FOUND,
            "application is not available for OIDC client management",
        ));
    }

    Ok(())
}

fn validate_redirect_uris(
    redirect_uris: &[String],
    allow_insecure_loopback_redirect_uris: bool,
) -> Result<(), Error> {
    let policy = if allow_insecure_loopback_redirect_uris {
        RedirectUriPolicy::web_with_insecure_loopback()
    } else {
        RedirectUriPolicy::web()
    };
    let mut unique = HashSet::with_capacity(redirect_uris.len());

    for redirect_uri in redirect_uris {
        validate_redirect_uri(redirect_uri, policy).map_err(|error| {
            Error::with_code(
                StatusCode::BAD_REQUEST,
                format!("invalid redirect URI: {error}"),
            )
        })?;

        if !unique.insert(redirect_uri.as_str()) {
            return Err(Error::with_code(
                StatusCode::BAD_REQUEST,
                "redirect URIs must be unique within a client",
            ));
        }
    }

    Ok(())
}

/// List OIDC clients registered to an application.
#[utoipa::path(
    get,
    path = "/tenants/{tenant_id}/applications/{application_id}/oidc-clients",
    tag = "OidcClients",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, body = ApiResponse<PagedResponse<OidcClientVO>>),
        (status = 203, description = "Missing Authorization header"),
        (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
        (status = 403, description = "Insufficient permission or tenant scope", body = ApiResponse<ErrorResponse>),
        (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_oidc_clients.list",
    skip(auth, database),
    fields(otel.kind = "internal", operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, page = field::Empty, per_page = field::Empty)
)]
pub async fn list_oidc_clients(
    auth: PlatformPermissionGuard<OidcClientRead>,
    app: ResolvedApplication,
    OptionalQuery(query): OptionalQuery<OidcClientsListQuery>,
    State(AppState { database, .. }): State<AppState>,
) -> AppResult<PagedResponse<OidcClientVO>> {
    ensure_oidc_management_application(&app)?;
    auth.ensure_tenant_scope(app.tenant_id(), &database).await?;

    let operator_id = auth.claim.sub;
    let application_id = app.id();
    let page = query.unwrap_or_default().into_page()?;
    Span::current().tap(|span| {
        span.record("operator_id", field::display(&operator_id))
            .record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("page", page.page)
            .record("per_page", page.per_page);
    });

    let PagedResponse { items, page_info } =
        OidcClients::get_clients(application_id, page, &database)
            .await
            .inspect_err(|error| {
                error!(%operator_id, %application_id, %error, "failed to list OIDC clients");
            })?;
    let client_ids = items.iter().map(|client| client.id).collect::<Vec<_>>();
    let redirect_uris = OidcClients::get_redirect_uris(client_ids, &database)
        .await
        .inspect_err(|error| {
            error!(%operator_id, %application_id, %error, "failed to list OIDC client redirect URIs");
        })?;
    let mut redirect_uris_by_client = HashMap::new();
    for redirect_uri in redirect_uris {
        redirect_uris_by_client
            .entry(redirect_uri.oidc_client_id)
            .or_insert_with(Vec::new)
            .push(redirect_uri);
    }

    let items = items
        .into_iter()
        .map(|client| {
            let redirect_uris = redirect_uris_by_client
                .remove(&client.id)
                .unwrap_or_default();
            crate::conversion::oidc_clients::oidc_client_model_to_vo(client, redirect_uris)
        })
        .collect();

    Ok(ApiResponse::new(PagedResponse { items, page_info }))
}

/// Register a public Web OIDC client to an application.
#[utoipa::path(
    post,
    path = "/tenants/{tenant_id}/applications/{application_id}/oidc-clients",
    tag = "OidcClients",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
    ),
    request_body = CreateOidcClientRequest,
    responses(
        (status = 200, body = ApiResponse<OidcClientVO>),
        (status = 203, description = "Missing Authorization header"),
        (status = 400, description = "Invalid token, client name, or redirect URI", body = ApiResponse<ErrorResponse>),
        (status = 403, description = "Insufficient permission or tenant scope", body = ApiResponse<ErrorResponse>),
        (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_oidc_clients.create",
    skip(auth, applications, auditing, database, name, redirect_uris),
    fields(otel.kind = "internal", operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, oidc_client_id = field::Empty, client_id = field::Empty)
)]
pub async fn create_oidc_client(
    auth: PlatformPermissionGuard<OidcClientCreate>,
    app: ResolvedApplication,
    State(AppState {
        applications,
        auditing,
        database,
        ..
    }): State<AppState>,
    Garde(Json(CreateOidcClientRequest {
        name,
        redirect_uris,
    })): Garde<Json<CreateOidcClientRequest>>,
) -> AppResult<OidcClientVO> {
    ensure_oidc_management_application(&app)?;
    auth.ensure_tenant_scope(app.tenant_id(), &database).await?;

    let operator_id = auth.claim.sub;
    let tenant_id = app.tenant_id();
    let application_id = app.id();
    let oidc_client_id = Uuid::now_v7();
    let client_id = Sqid::from(oidc_client_id).into_inner();
    Span::current().tap(|span| {
        span.record("operator_id", field::display(&operator_id))
            .record("tenant_id", field::display(&tenant_id))
            .record("application_id", field::display(&application_id))
            .record("oidc_client_id", field::display(&oidc_client_id))
            .record("client_id", field::display(&client_id));
    });

    let configuration = applications.get_configuration(application_id).await?;
    validate_redirect_uris(
        &redirect_uris,
        configuration.oidc.allow_insecure_loopback_redirect_uris,
    )?;

    let transaction = database.begin().await.inspect_err(|error| {
        error!(%operator_id, %application_id, %error, "failed to begin OIDC client transaction");
    })?;
    let client = OidcClients::create_client(
        oidc_client_id,
        application_id,
        client_id.clone(),
        name.clone(),
        &transaction,
    )
    .await
    .inspect_err(|error| {
        error!(%operator_id, %application_id, %oidc_client_id, %error, "failed to create OIDC client");
    })?;
    let redirect_uri_models =
        OidcClients::create_redirect_uris(oidc_client_id, redirect_uris, &transaction)
            .await
            .inspect_err(|error| {
                error!(%operator_id, %application_id, %oidc_client_id, %error, "failed to create OIDC client redirect URIs");
            })?;
    transaction.commit().await.inspect_err(|error| {
        error!(%operator_id, %application_id, %oidc_client_id, %error, "failed to commit OIDC client transaction");
    })?;

    info!(%operator_id, %application_id, %oidc_client_id, %client_id, "OIDC client created successfully");

    auditing
        .write(AuditPayload::from(CreateOidcClientPayload {
            operator_id,
            tenant_id,
            application_id,
            oidc_client_id,
            client_id,
            name,
        }))
        .await;

    Ok(ApiResponse::new(
        crate::conversion::oidc_clients::oidc_client_model_to_vo(client, redirect_uri_models),
    ))
}

/// Get one OIDC client registered to an application.
#[utoipa::path(
    get,
    path = "/tenants/{tenant_id}/applications/{application_id}/oidc-clients/{client_id}",
    tag = "OidcClients",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("client_id" = String, Path, description = "OIDC client ID"),
    ),
    responses(
        (status = 200, body = ApiResponse<OidcClientVO>),
        (status = 203, description = "Missing Authorization header"),
        (status = 400, description = "Invalid token or path", body = ApiResponse<ErrorResponse>),
        (status = 403, description = "Insufficient permission or tenant scope", body = ApiResponse<ErrorResponse>),
        (status = 404, description = "Application or OIDC client not found", body = ApiResponse<ErrorResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_oidc_clients.get",
    skip(auth, database, client_id),
    fields(otel.kind = "internal", operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, client_id = field::Empty)
)]
pub async fn get_oidc_client(
    auth: PlatformPermissionGuard<OidcClientRead>,
    app: ResolvedApplication,
    State(AppState { database, .. }): State<AppState>,
    Path((_tenant_id, _application_id, client_id)): Path<(Sqid, Sqid, String)>,
) -> AppResult<OidcClientVO> {
    ensure_oidc_management_application(&app)?;
    auth.ensure_tenant_scope(app.tenant_id(), &database).await?;

    let operator_id = auth.claim.sub;
    let application_id = app.id();
    Span::current().tap(|span| {
        span.record("operator_id", field::display(&operator_id))
            .record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("client_id", field::display(&client_id));
    });

    let client = OidcClients::get_client(application_id, &client_id, &database)
        .await
        .inspect_err(|error| {
            error!(%operator_id, %application_id, %error, "failed to get OIDC client");
        })?;
    let redirect_uris = OidcClients::get_redirect_uris(vec![client.id], &database)
        .await
        .inspect_err(|error| {
            error!(%operator_id, %application_id, %error, "failed to get OIDC client redirect URIs");
        })?;

    Ok(ApiResponse::new(
        crate::conversion::oidc_clients::oidc_client_model_to_vo(client, redirect_uris),
    ))
}
