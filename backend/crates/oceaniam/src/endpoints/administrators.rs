//! Administrator management-related API endpoints
//!
//! NOTE: This package is allowed to access the database directly without going through any cache
//! layer.

use axum::{
    Json,
    extract::{Path, State},
};
use axum_extra::extract::OptionalQuery;
use axum_valid::Garde;
use oceaniam_audit::types::{AuditPayload, CreateAdministratorPayload, PatchAdministratorPayload};
use oceaniam_common::{
    ApiResponse, ErrorResponse, PageParam, PagedResponse, RestResult, error::Error,
    jwt::SystemClaim, types::sqid::Sqid,
};
use oceaniam_database::{
    helper::{
        SafeTransactionConnectionTrait,
        administrators::{AdministratorsHelper, UpdateAdministratorModel},
    },
    model::{administrators, prelude::*},
};
use oceaniam_vo::administrators::{
    AdministratorVO, CreateAdministratorRequest, CreateAdministratorResponse,
    PatchAdministratorRequest,
};
use sea_orm::TransactionTrait;
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    middlewares::auth::RequireAuth,
    state::{AppState, credentials::ManagedCredentialVaults},
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(get_administrators))
        .routes(routes!(create_administrator))
        .routes(routes!(patch_administrator))
}

/// Get administrator list
#[utoipa::path(
        get,
        path = "/administrators",
        tag = "Administrators",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("page" = Option<u64>, Query, description = "Page number"),
            ("per_page" = Option<u64>, Query, description = "Items per page"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<AdministratorVO>>),
            (status = 401, description = "Unauthorized"),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "administrators.list",
    skip(database),
    fields(page = field::Empty, per_page = field::Empty)
)]
pub async fn get_administrators(
    _: RequireAuth<SystemClaim>,
    OptionalQuery(query): OptionalQuery<PageParam>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> RestResult<PagedResponse<AdministratorVO>> {
    let page = query.unwrap_or_default();
    Span::current().tap(|it| {
        it.record("page", page.page)
            .record("per_page", page.per_page);
    });

    let PagedResponse { items, page_info } = Administrators::get_administrators(page, &database)
        .await
        .inspect_err(|e| error!(error = %e, "administrator list query failed"))?;
    let items = items.into_iter().map(AdministratorVO::from).collect();

    Ok(ApiResponse::new(PagedResponse { items, page_info }))
}

/// Create administrator
#[utoipa::path(
        post,
        path = "/administrators",
        tag = "Administrators",
        params(("Authorization" = String, Header, description = "Bearer token")),
        request_body = CreateAdministratorRequest,
        responses(
            (status = 200, body = ApiResponse<CreateAdministratorResponse>),
            (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 409, description = "Administrator already exists", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "administrators.create",
    skip(auth, database, credentials, auditing, name),
    fields(operator_id = field::Empty, administrator_id = field::Empty)
)]
pub async fn create_administrator(
    auth: RequireAuth<SystemClaim>,
    State(AppState {
        database,
        credentials,
        auditing,
        ..
    }): State<AppState<'_>>,
    Garde(Json(CreateAdministratorRequest { name })): Garde<Json<CreateAdministratorRequest>>,
) -> RestResult<CreateAdministratorResponse> {
    let operator_id = auth.token.claims.sub;
    let administrator_id = Uuid::now_v7();
    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id))
            .record("administrator_id", field::display(&administrator_id));
    });

    if Administrators::get_by_name(&name, &database).await.is_ok() {
        return Err(Error::with_code(
            axum::http::StatusCode::CONFLICT,
            format!("administrator name={name} already exists"),
        ));
    }

    let initial_password = oceaniam_common::helpers::gen_random(24);
    let transaction = database.begin().await?;

    if let Err(error) = credentials
        .create_with_password_in_tx(administrator_id, &initial_password, &transaction)
        .await
    {
        error!(
            %operator_id,
            %administrator_id,
            %error,
            "administrator credential creation failed"
        );
        return Err(error);
    }

    let administrator =
        match Administrators::create_administrator(administrator_id, &name, &transaction).await {
            Ok(administrator) => administrator,
            Err(error) => {
                error!(
                    %operator_id,
                    %administrator_id,
                    %error,
                    "administrator creation failed"
                );
                return Err(error);
            }
        };

    transaction.commit().await?;

    info!(
        %operator_id,
        %administrator_id,
        "administrator created successfully"
    );

    auditing
        .write(AuditPayload::from(CreateAdministratorPayload {
            administrator_id,
            operator_id,
            name: administrator.name.clone(),
        }))
        .await;

    Ok(ApiResponse::new(CreateAdministratorResponse {
        administrator: administrator.into(),
        initial_password,
    }))
}

/// Update administrator
#[utoipa::path(
        patch,
        path = "/administrators/{target_id}",
        tag = "Administrators",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("administrator_id", description = "Administrator ID"),
        ),
        request_body = PatchAdministratorRequest,
        responses(
            (status = 200, body = ApiResponse<AdministratorVO>),
            (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "Administrator not found", body = ApiResponse<ErrorResponse>),
            (status = 409, description = "Administrator already exists", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "administrators.patch",
    skip(auth, database, auditing, target_id, payload),
    fields(operator_id = field::Empty, administrator_id = field::Empty)
)]
pub async fn patch_administrator(
    auth: RequireAuth<SystemClaim>,
    Path(target_id): Path<Sqid>,
    State(AppState {
        database,
        auditing,
        credentials,
        ..
    }): State<AppState<'_>>,
    Garde(Json(payload)): Garde<Json<PatchAdministratorRequest>>,
) -> RestResult<AdministratorVO> {
    let operator_id = auth.token.claims.sub;
    let target_id: Uuid = target_id.try_into()?;
    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id))
            .record("target_id", field::display(&target_id));
    });

    let transaction = database.begin().await?;

    // NOTE: Prepare for future.
    //
    // HOLY SHIT WHERE's MY POLICY ENGINE?
    let target_administrator = if operator_id == target_id {
        patch_administrator_self(operator_id, payload.clone(), &credentials, &transaction).await
    } else {
        patch_administrator_other(
            operator_id,
            target_id,
            payload.clone(),
            &credentials,
            &transaction,
        )
        .await
    }?;

    transaction.commit().await?;

    info!(
        %operator_id,
        %target_id,
        "administrator updated successfully"
    );

    let PatchAdministratorRequest { name, password } = payload;

    auditing
        .write(AuditPayload::from(PatchAdministratorPayload {
            target_id,
            operator_id,
            name,
            password: password.map(|it| "*".repeat(it.len())),
        }))
        .await;

    Ok(ApiResponse::new(target_administrator.into()))
}

#[tracing::instrument(
    level = "info",
    name = "administrators.patch.self",
    skip(transaction, name),
    fields(%operator_id)
)]
async fn patch_administrator_self(
    operator_id: Uuid,
    PatchAdministratorRequest { name, password }: PatchAdministratorRequest,
    state_credentials: &ManagedCredentialVaults,
    transaction: &impl SafeTransactionConnectionTrait,
) -> Result<administrators::Model, Error> {
    // TODO: OK WE NEED POLICY ENGINE.
    patch_administrator_other(
        operator_id,
        operator_id,
        PatchAdministratorRequest { name, password },
        state_credentials,
        transaction,
    )
    .await
}

#[tracing::instrument(
    level = "info",
    name = "administrators.patch.other",
    skip(transaction, name),
    fields(%operator_id, %target_id)
)]
async fn patch_administrator_other(
    operator_id: Uuid,
    target_id: Uuid,
    PatchAdministratorRequest { name, password }: PatchAdministratorRequest,
    state_credentials: &ManagedCredentialVaults,
    transaction: &impl SafeTransactionConnectionTrait,
) -> Result<administrators::Model, Error> {
    if let Some(password) = password {
        // TOO EXPENSIVE.
        state_credentials
            .update_password_in_tx(target_id, password, transaction)
            .await?;
    }

    Administrators::update_model(target_id, UpdateAdministratorModel { name }, transaction)
        .await
        .inspect_err(|e| {
            error!(
                %target_id,
                error = %e,
                "administrator patch for other administrators failed"
            )
        })
}
