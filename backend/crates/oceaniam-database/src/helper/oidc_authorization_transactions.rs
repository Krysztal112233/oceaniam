use axum::http::StatusCode;
use chrono::{DateTime, FixedOffset};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait, ExprTrait, QueryFilter,
    QuerySelect, Statement, TransactionSession,
    sea_query::Expr,
};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::SafeTransactionConnectionTrait,
    model::{
        self,
        prelude::OidcAuthorizationTransactions,
        sea_orm_active_enums::{OidcAuthorizationTransactionStatus, OidcPkceMethod},
    },
};

/// Validated, immutable request data captured when an authorization flow starts.
///
/// This deliberately does not implement `Debug`: it contains digests and protocol values that
/// must not be recorded by generic diagnostics.
pub struct CreateAuthorizationTransactionInput {
    pub tenant_id: Uuid,
    pub application_id: Uuid,
    pub oidc_client_id: Uuid,
    pub issuer: String,
    pub redirect_uri: String,
    pub state: String,
    pub nonce: Option<String>,
    pub code_challenge: String,
    pub browser_binding_digest: [u8; 32],
    pub csrf_digest: [u8; 32],
}

/// Validated transition data for a password-authenticated authorization transaction.
///
/// This deliberately does not implement `Debug`: it contains digests that must not be recorded
/// by generic diagnostics.
pub struct AuthenticateAuthorizationTransactionInput {
    pub id: Uuid,
    pub browser_binding_digest: [u8; 32],
    pub csrf_digest: [u8; 32],
    pub expected_revision: i64,
    pub subject_id: Uuid,
}

/// Validated transition data that binds an MFA challenge to a pending transaction.
///
/// This deliberately does not implement `Debug`: it contains digests that must not be recorded
/// by generic diagnostics.
pub struct ChallengeAuthorizationTransactionInput {
    pub id: Uuid,
    pub browser_binding_digest: [u8; 32],
    pub csrf_digest: [u8; 32],
    pub expected_revision: i64,
    pub challenge_id: Uuid,
}

/// Validated transition data for a challenge-authenticated authorization transaction.
///
/// This deliberately does not implement `Debug`: it contains digests that must not be recorded
/// by generic diagnostics.
pub struct AuthenticateChallengedAuthorizationTransactionInput {
    pub id: Uuid,
    pub browser_binding_digest: [u8; 32],
    pub csrf_digest: [u8; 32],
    pub expected_revision: i64,
    pub challenge_id: Uuid,
    pub subject_id: Uuid,
}

#[async_trait::async_trait]
pub trait OidcAuthorizationTransactionsHelper {
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_transactions.create",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn create_authorization_transaction(
        CreateAuthorizationTransactionInput {
            tenant_id,
            application_id,
            oidc_client_id,
            issuer,
            redirect_uri,
            state,
            nonce,
            code_challenge,
            browser_binding_digest,
            csrf_digest,
        }: CreateAuthorizationTransactionInput,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_authorization_transactions::Model, Error> {
        model::oidc_authorization_transactions::ActiveModel {
            id: Set(Uuid::now_v7()),
            tenant_id: Set(tenant_id),
            application_id: Set(application_id),
            oidc_client_id: Set(oidc_client_id),
            issuer: Set(issuer),
            redirect_uri: Set(redirect_uri),
            requested_scope: Set("openid".to_owned()),
            state: Set(state),
            nonce: Set(nonce),
            code_challenge: Set(code_challenge),
            code_challenge_method: Set(OidcPkceMethod::S256),
            browser_binding_digest: Set(browser_binding_digest.to_vec()),
            csrf_digest: Set(csrf_digest.to_vec()),
            status: NotSet,
            revision: NotSet,
            terminal_at: NotSet,
            created_at: NotSet,
            expires_at: NotSet,
            subject_id: NotSet,
            authenticated_at: NotSet,
            challenge_id: NotSet,
        }
        .insert(database)
        .await
        .map_err(|_| authorization_transaction_storage_error())
    }

    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_transactions.get_active",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_active_authorization_transaction(
        id: Uuid,
        browser_binding_digest: [u8; 32],
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_authorization_transactions::Model, Error> {
        use model::oidc_authorization_transactions::Column::*;

        OidcAuthorizationTransactions::find()
            .filter(Id.eq(id))
            .filter(BrowserBindingDigest.eq(browser_binding_digest.to_vec()))
            .filter(Status.eq(OidcAuthorizationTransactionStatus::Pending))
            .filter(TerminalAt.is_null())
            .filter(Expr::col(ExpiresAt).gt(Expr::cust("clock_timestamp()")))
            .one(database)
            .await
            .map_err(|_| authorization_transaction_storage_error())?
            .ok_or_else(authorization_transaction_unavailable)
    }

    /// Reads a still-usable login candidate by its full browser and CSRF binding.
    ///
    /// Every binding, tenant-ownership, status, and database-clock expiry predicate shares one
    /// filtered read so a missing cookie, wrong binding, wrong CSRF, expired, or non-pending
    /// transaction is indistinguishable from an absent one. This is an unlocked pre-gate; the
    /// authoritative check is [`Self::authenticate_authorization_transaction`].
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_transactions.get_for_login",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_login_authorization_transaction(
        id: Uuid,
        tenant_id: Uuid,
        browser_binding_digest: [u8; 32],
        csrf_digest: [u8; 32],
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_authorization_transactions::Model, Error> {
        use model::oidc_authorization_transactions::Column::*;

        OidcAuthorizationTransactions::find()
            .filter(Id.eq(id))
            .filter(TenantId.eq(tenant_id))
            .filter(BrowserBindingDigest.eq(browser_binding_digest.to_vec()))
            .filter(CsrfDigest.eq(csrf_digest.to_vec()))
            .filter(Status.eq(OidcAuthorizationTransactionStatus::Pending))
            .filter(TerminalAt.is_null())
            .filter(Expr::col(ExpiresAt).gt(Expr::cust("clock_timestamp()")))
            .one(database)
            .await
            .map_err(|_| authorization_transaction_storage_error())?
            .ok_or_else(authorization_transaction_unavailable)
    }

    /// Reads a still-live transaction by its full browser and CSRF binding for the challenge
    /// step, without any status predicate.
    ///
    /// Every binding, tenant-ownership, and database-clock expiry predicate shares one filtered
    /// read so a missing cookie, wrong binding, wrong CSRF, expired, or absent transaction is
    /// indistinguishable. The status is deliberately left to the caller: the challenge endpoint
    /// maps a live transaction in the wrong state to the uniform credential-failure page so the
    /// state-machine position never leaks, while the login endpoint re-renders the challenge
    /// form only for `awaiting_challenge`.
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_transactions.get_for_challenge",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_challenged_authorization_transaction(
        id: Uuid,
        tenant_id: Uuid,
        browser_binding_digest: [u8; 32],
        csrf_digest: [u8; 32],
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_authorization_transactions::Model, Error> {
        use model::oidc_authorization_transactions::Column::*;

        OidcAuthorizationTransactions::find()
            .filter(Id.eq(id))
            .filter(TenantId.eq(tenant_id))
            .filter(BrowserBindingDigest.eq(browser_binding_digest.to_vec()))
            .filter(CsrfDigest.eq(csrf_digest.to_vec()))
            .filter(TerminalAt.is_null())
            .filter(Expr::col(ExpiresAt).gt(Expr::cust("clock_timestamp()")))
            .one(database)
            .await
            .map_err(|_| authorization_transaction_storage_error())?
            .ok_or_else(authorization_transaction_unavailable)
    }

    /// Transitions `pending -> authenticated`, recording the authenticated subject.
    ///
    /// The caller owns the surrounding transaction (live registration revalidation, this
    /// transition, and the success audit commit atomically). The row is locked `FOR UPDATE`
    /// only after all binding and optimistic-concurrency predicates match; expiry is then
    /// re-sampled with `clock_timestamp()` so time spent waiting for the lock cannot revive an
    /// expired transaction. The conditional update returns exactly one concurrent winner.
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_transactions.authenticate",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn authenticate_authorization_transaction(
        AuthenticateAuthorizationTransactionInput {
            id,
            browser_binding_digest,
            csrf_digest,
            expected_revision,
            subject_id,
        }: AuthenticateAuthorizationTransactionInput,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_authorization_transactions::Model, Error> {
        use model::oidc_authorization_transactions::Column::*;

        let candidate = OidcAuthorizationTransactions::find()
            .filter(Id.eq(id))
            .filter(BrowserBindingDigest.eq(browser_binding_digest.to_vec()))
            .filter(CsrfDigest.eq(csrf_digest.to_vec()))
            .filter(Status.eq(OidcAuthorizationTransactionStatus::Pending))
            .filter(TerminalAt.is_null())
            .filter(Revision.eq(expected_revision))
            .lock_exclusive()
            .one(database)
            .await
            .map_err(|_| authorization_transaction_storage_error())?
            .ok_or_else(authorization_transaction_unavailable)?;

        // PostgreSQL's CURRENT_TIMESTAMP is fixed at transaction start. Sample the wall clock
        // only after acquiring the row lock so time spent waiting cannot revive an expired
        // transaction.
        let now: DateTime<FixedOffset> = database
            .query_one_raw(Statement::from_string(
                DatabaseBackend::Postgres,
                "SELECT clock_timestamp() AS now".to_owned(),
            ))
            .await
            .map_err(|_| authorization_transaction_storage_error())?
            .ok_or_else(authorization_transaction_storage_error)?
            .try_get("", "now")
            .map_err(|_| authorization_transaction_storage_error())?;
        if candidate.expires_at <= now {
            return Err(authorization_transaction_unavailable());
        }

        let mut updated = OidcAuthorizationTransactions::update_many()
            .col_expr(
                Status,
                Expr::value(OidcAuthorizationTransactionStatus::Authenticated),
            )
            .col_expr(SubjectId, Expr::value(subject_id))
            .col_expr(AuthenticatedAt, Expr::value(now))
            .col_expr(Revision, Expr::col(Revision).add(1))
            .filter(Id.eq(candidate.id))
            .filter(Status.eq(OidcAuthorizationTransactionStatus::Pending))
            .filter(TerminalAt.is_null())
            .filter(Revision.eq(expected_revision))
            .exec_with_returning(database)
            .await
            .map_err(|_| authorization_transaction_storage_error())?;

        debug_assert!(
            updated.len() <= 1,
            "a primary-key-filtered update returned multiple OIDC authorization transactions"
        );
        updated
            .pop()
            .ok_or_else(authorization_transaction_unavailable)
    }

    /// Transitions `pending -> awaiting_challenge`, binding the minted MFA challenge.
    ///
    /// The caller owns the surrounding transaction (live registration revalidation, challenge
    /// creation, this transition, and the challenge-create audit commit atomically). Locking,
    /// post-lock expiry re-sampling, and the exactly-one-winner conditional update mirror
    /// [`Self::authenticate_authorization_transaction`].
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_transactions.challenge",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn challenge_authorization_transaction(
        ChallengeAuthorizationTransactionInput {
            id,
            browser_binding_digest,
            csrf_digest,
            expected_revision,
            challenge_id,
        }: ChallengeAuthorizationTransactionInput,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_authorization_transactions::Model, Error> {
        use model::oidc_authorization_transactions::Column::*;

        let candidate = OidcAuthorizationTransactions::find()
            .filter(Id.eq(id))
            .filter(BrowserBindingDigest.eq(browser_binding_digest.to_vec()))
            .filter(CsrfDigest.eq(csrf_digest.to_vec()))
            .filter(Status.eq(OidcAuthorizationTransactionStatus::Pending))
            .filter(TerminalAt.is_null())
            .filter(Revision.eq(expected_revision))
            .lock_exclusive()
            .one(database)
            .await
            .map_err(|_| authorization_transaction_storage_error())?
            .ok_or_else(authorization_transaction_unavailable)?;

        // PostgreSQL's CURRENT_TIMESTAMP is fixed at transaction start. Sample the wall clock
        // only after acquiring the row lock so time spent waiting cannot revive an expired
        // transaction.
        let now: DateTime<FixedOffset> = database
            .query_one_raw(Statement::from_string(
                DatabaseBackend::Postgres,
                "SELECT clock_timestamp() AS now".to_owned(),
            ))
            .await
            .map_err(|_| authorization_transaction_storage_error())?
            .ok_or_else(authorization_transaction_storage_error)?
            .try_get("", "now")
            .map_err(|_| authorization_transaction_storage_error())?;
        if candidate.expires_at <= now {
            return Err(authorization_transaction_unavailable());
        }

        let mut updated = OidcAuthorizationTransactions::update_many()
            .col_expr(
                Status,
                Expr::value(OidcAuthorizationTransactionStatus::AwaitingChallenge),
            )
            .col_expr(ChallengeId, Expr::value(challenge_id))
            .col_expr(Revision, Expr::col(Revision).add(1))
            .filter(Id.eq(candidate.id))
            .filter(Status.eq(OidcAuthorizationTransactionStatus::Pending))
            .filter(TerminalAt.is_null())
            .filter(Revision.eq(expected_revision))
            .exec_with_returning(database)
            .await
            .map_err(|_| authorization_transaction_storage_error())?;

        debug_assert!(
            updated.len() <= 1,
            "a primary-key-filtered update returned multiple OIDC authorization transactions"
        );
        updated
            .pop()
            .ok_or_else(authorization_transaction_unavailable)
    }

    /// Transitions `awaiting_challenge -> authenticated`, recording the authenticated subject.
    ///
    /// The transition only proceeds when the supplied `challenge_id` matches the bound column,
    /// so the challenge row consumed by the caller is the exact challenge this transaction
    /// awaits. Locking, post-lock expiry re-sampling, and the exactly-one-winner conditional
    /// update mirror [`Self::authenticate_authorization_transaction`].
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_transactions.authenticate_challenged",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn authenticate_challenged_authorization_transaction(
        AuthenticateChallengedAuthorizationTransactionInput {
            id,
            browser_binding_digest,
            csrf_digest,
            expected_revision,
            challenge_id,
            subject_id,
        }: AuthenticateChallengedAuthorizationTransactionInput,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_authorization_transactions::Model, Error> {
        use model::oidc_authorization_transactions::Column::*;

        let candidate = OidcAuthorizationTransactions::find()
            .filter(Id.eq(id))
            .filter(BrowserBindingDigest.eq(browser_binding_digest.to_vec()))
            .filter(CsrfDigest.eq(csrf_digest.to_vec()))
            .filter(Status.eq(OidcAuthorizationTransactionStatus::AwaitingChallenge))
            .filter(TerminalAt.is_null())
            .filter(Revision.eq(expected_revision))
            .filter(ChallengeId.eq(challenge_id))
            .lock_exclusive()
            .one(database)
            .await
            .map_err(|_| authorization_transaction_storage_error())?
            .ok_or_else(authorization_transaction_unavailable)?;

        // PostgreSQL's CURRENT_TIMESTAMP is fixed at transaction start. Sample the wall clock
        // only after acquiring the row lock so time spent waiting cannot revive an expired
        // transaction.
        let now: DateTime<FixedOffset> = database
            .query_one_raw(Statement::from_string(
                DatabaseBackend::Postgres,
                "SELECT clock_timestamp() AS now".to_owned(),
            ))
            .await
            .map_err(|_| authorization_transaction_storage_error())?
            .ok_or_else(authorization_transaction_storage_error)?
            .try_get("", "now")
            .map_err(|_| authorization_transaction_storage_error())?;
        if candidate.expires_at <= now {
            return Err(authorization_transaction_unavailable());
        }

        let mut updated = OidcAuthorizationTransactions::update_many()
            .col_expr(
                Status,
                Expr::value(OidcAuthorizationTransactionStatus::Authenticated),
            )
            .col_expr(SubjectId, Expr::value(subject_id))
            .col_expr(AuthenticatedAt, Expr::value(now))
            .col_expr(Revision, Expr::col(Revision).add(1))
            .filter(Id.eq(candidate.id))
            .filter(Status.eq(OidcAuthorizationTransactionStatus::AwaitingChallenge))
            .filter(TerminalAt.is_null())
            .filter(Revision.eq(expected_revision))
            .filter(ChallengeId.eq(challenge_id))
            .exec_with_returning(database)
            .await
            .map_err(|_| authorization_transaction_storage_error())?;

        debug_assert!(
            updated.len() <= 1,
            "a primary-key-filtered update returned multiple OIDC authorization transactions"
        );
        updated
            .pop()
            .ok_or_else(authorization_transaction_unavailable)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_transactions.cancel",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn cancel_authorization_transaction(
        id: Uuid,
        browser_binding_digest: [u8; 32],
        csrf_digest: [u8; 32],
        expected_revision: i64,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_authorization_transactions::Model, Error> {
        use model::oidc_authorization_transactions::Column::*;

        let transaction = database
            .begin()
            .await
            .map_err(|_| authorization_transaction_storage_error())?;

        let result = async {
            let candidate = OidcAuthorizationTransactions::find()
                .filter(Id.eq(id))
                .filter(BrowserBindingDigest.eq(browser_binding_digest.to_vec()))
                .filter(CsrfDigest.eq(csrf_digest.to_vec()))
                .filter(Status.eq(OidcAuthorizationTransactionStatus::Pending))
                .filter(TerminalAt.is_null())
                .filter(Revision.eq(expected_revision))
                .lock_exclusive()
                .one(&transaction)
                .await
                .map_err(|_| authorization_transaction_storage_error())?
                .ok_or_else(authorization_transaction_unavailable)?;

            // PostgreSQL's CURRENT_TIMESTAMP is fixed at transaction start. Sample the wall clock
            // only after acquiring the row lock so time spent waiting cannot revive an expired
            // transaction.
            let now: DateTime<FixedOffset> = transaction
                .query_one_raw(Statement::from_string(
                    DatabaseBackend::Postgres,
                    "SELECT clock_timestamp() AS now".to_owned(),
                ))
                .await
                .map_err(|_| authorization_transaction_storage_error())?
                .ok_or_else(authorization_transaction_storage_error)?
                .try_get("", "now")
                .map_err(|_| authorization_transaction_storage_error())?;
            if candidate.expires_at <= now {
                return Err(authorization_transaction_unavailable());
            }

            let mut updated = OidcAuthorizationTransactions::update_many()
                .col_expr(
                    Status,
                    Expr::value(OidcAuthorizationTransactionStatus::Cancelled),
                )
                .col_expr(TerminalAt, Expr::value(now))
                .col_expr(Revision, Expr::col(Revision).add(1))
                .filter(Id.eq(candidate.id))
                .filter(Status.eq(OidcAuthorizationTransactionStatus::Pending))
                .filter(TerminalAt.is_null())
                .filter(Revision.eq(expected_revision))
                .exec_with_returning(&transaction)
                .await
                .map_err(|_| authorization_transaction_storage_error())?;

            debug_assert!(
                updated.len() <= 1,
                "a primary-key-filtered update returned multiple OIDC authorization transactions"
            );
            updated
                .pop()
                .ok_or_else(authorization_transaction_unavailable)
        }
        .await;

        match result {
            Ok(updated) => {
                transaction
                    .commit()
                    .await
                    .map_err(|_| authorization_transaction_storage_error())?;
                Ok(updated)
            }
            Err(error) => {
                transaction
                    .rollback()
                    .await
                    .map_err(|_| authorization_transaction_storage_error())?;
                Err(error)
            }
        }
    }
}

fn authorization_transaction_unavailable() -> Error {
    Error::with_code(
        StatusCode::NOT_FOUND,
        "OIDC authorization transaction is unavailable",
    )
}

fn authorization_transaction_storage_error() -> Error {
    Error::with_code(
        StatusCode::INTERNAL_SERVER_ERROR,
        "OIDC authorization transaction storage operation failed",
    )
}

impl OidcAuthorizationTransactionsHelper for OidcAuthorizationTransactions {}
