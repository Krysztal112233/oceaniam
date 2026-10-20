use axum::http::StatusCode;
use chrono::Duration;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, ExprTrait, QueryFilter,
    sea_query::Expr,
};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::{SafeTransactionConnectionTrait, db_clock_now, exactly_one_returned},
    model::{self, prelude::OidcAuthorizationCodes},
};

/// Validated issuance data for a freshly authenticated authorization code.
///
/// This deliberately does not implement `Debug`: it contains a code digest that must not be
/// recorded by generic diagnostics.
pub struct IssueAuthorizationCodeInput {
    pub transaction_id: Uuid,
    pub code_digest: [u8; 32],
    pub redirect_uri: String,
    pub subject_id: Uuid,
}

#[async_trait::async_trait]
pub trait OidcAuthorizationCodesHelper {
    /// Issues an authorization code row anchored on one database-clock sample (Rule 3), so the
    /// row invariant `expires_at - created_at == ttl` holds exactly. Only the digest is stored;
    /// the plaintext code never reaches the database.
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_codes.issue",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn issue_authorization_code(
        IssueAuthorizationCodeInput {
            transaction_id,
            code_digest,
            redirect_uri,
            subject_id,
        }: IssueAuthorizationCodeInput,
        ttl: Duration,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_authorization_codes::Model, Error> {
        let now = db_clock_now(database)
            .await
            .map_err(|_| authorization_code_storage_error())?;

        model::oidc_authorization_codes::ActiveModel {
            id: Set(Uuid::now_v7()),
            transaction_id: Set(transaction_id),
            code_digest: Set(code_digest.to_vec()),
            redirect_uri: Set(redirect_uri),
            subject_id: Set(subject_id),
            expires_at: Set(now + ttl),
            redeemed_at: Set(None),
            created_at: Set(now),
        }
        .insert(database)
        .await
        .map_err(|_| authorization_code_storage_error())
    }

    /// Redeems an authorization code exactly once.
    ///
    /// `code_digest` is the row's UNIQUE key, so filtering on it identifies at most one row —
    /// the Rule 2 primary-key-equivalent shape permitted for unique keys. The conditional
    /// update re-checks the unredeemed state and the database-clock expiry, so a redeemed,
    /// expired, or absent code all fail with the same not-found error and a code can never be
    /// redeemed twice.
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_codes.consume",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn consume_authorization_code(
        code_digest: [u8; 32],
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_authorization_codes::Model, Error> {
        use model::oidc_authorization_codes::Column::*;

        let updated = OidcAuthorizationCodes::update_many()
            .filter(CodeDigest.eq(code_digest.to_vec()))
            .filter(RedeemedAt.is_null())
            .filter(Expr::col(ExpiresAt).gt(Expr::cust("clock_timestamp()")))
            .col_expr(RedeemedAt, Expr::cust("clock_timestamp()"))
            .exec_with_returning(database)
            .await
            .map_err(|_| authorization_code_storage_error())?;

        exactly_one_returned(updated, authorization_code_unavailable)
    }
}

fn authorization_code_unavailable() -> Error {
    Error::with_code(
        StatusCode::NOT_FOUND,
        "OIDC authorization code is unavailable",
    )
}

fn authorization_code_storage_error() -> Error {
    Error::with_code(
        StatusCode::INTERNAL_SERVER_ERROR,
        "OIDC authorization code storage operation failed",
    )
}

impl OidcAuthorizationCodesHelper for OidcAuthorizationCodes {}
