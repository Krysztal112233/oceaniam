//! Administrator management-related API endpoints

use axum::{Json, extract::State};
use axum_valid::Garde;
use oceaniam_audit::types::{AuditPayload, CreateAdministratorPayload};
use oceaniam_common::{
    ApiResponse, ErrorResponse, PagedResponse, RestResult, error::Error, jwt::SystemClaim,
};
use oceaniam_database::{helper::administrators::AdministratorsHelper, model::prelude::*};
use oceaniam_vo::administrators::{
    AdministratorVO, CreateAdministratorRequest, CreateAdministratorResponse,
};
use sea_orm::TransactionTrait;
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{middlewares::auth::RequireAuth, state::AppState};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(get_administrators))
        .routes(routes!(create_administrator))
}

/// Get administrator list
#[utoipa::path(
        get,
        path = "/administrators",
        tag = "Administrators",
        params(("Authorization" = String, Header, description = "Bearer token")),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<AdministratorVO>>),
            (status = 401, description = "Unauthorized"),
        ),
    )]
#[tracing::instrument(level = "info", name = "administrators.list", skip(database))]
pub async fn get_administrators(
    _: RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> RestResult<PagedResponse<AdministratorVO>> {
    let items: Vec<AdministratorVO> = Administrators::get_all(&database)
        .await
        .inspect_err(|e| error!(error = %e, "administrator list query failed"))?
        .into_iter()
        .map(AdministratorVO::from)
        .collect();

    Ok(ApiResponse::new(PagedResponse::with_entire(items)))
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

    let administrator = match Administrators::create(administrator_id, &name, &transaction).await {
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
