use oceaniam_common::consts;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::SafeTransactionConnectionTrait,
    model::{
        self,
        prelude::{Applications, OidcClientRedirectUris, OidcClients, Tenants},
    },
};

/// Untrusted identifiers discovered before the authorization lock sequence.
pub struct AuthorizationEntryCandidate {
    pub application_id: Uuid,
    pub oidc_client_id: Uuid,
}

/// Result of the preliminary tenant/client ownership lookup.
pub enum AuthorizationEntryResolution {
    TenantUnavailable,
    ClientUnavailable,
    Candidate(AuthorizationEntryCandidate),
}

/// Live registration state protected by shared locks through snapshot insertion.
pub struct LockedAuthorizationEntry {
    pub application_id: Uuid,
    pub oidc_client_id: Uuid,
    pub application_configuration: serde_json::Value,
}

/// Result of revalidating the preliminary relationship under ordered shared locks.
pub enum AuthorizationEntryLockResult {
    TenantUnavailable,
    RelationshipUnavailable,
    Locked(LockedAuthorizationEntry),
}

#[async_trait::async_trait]
pub trait OidcAuthorizationEntryHelper {
    /// Resolves only enough ownership state to identify rows for ordered locking. This result is
    /// not callback trust and must never authorize a redirect or snapshot by itself.
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_entry.resolve_candidate",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn resolve_authorization_entry_candidate(
        tenant_id: Uuid,
        client_id: &str,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<AuthorizationEntryResolution, Error> {
        if tenant_id == consts::SYSTEM_TENANT_UUID
            || Tenants::find_by_id(tenant_id)
                .one(database)
                .await?
                .is_none()
        {
            return Ok(AuthorizationEntryResolution::TenantUnavailable);
        }

        let Some(client) = OidcClients::find()
            .filter(model::oidc_clients::Column::ClientId.eq(client_id))
            .one(database)
            .await?
        else {
            return Ok(AuthorizationEntryResolution::ClientUnavailable);
        };

        let application_exists = Applications::find_by_id(client.application_id)
            .filter(model::applications::Column::TenantId.eq(tenant_id))
            .one(database)
            .await?
            .is_some();
        if !application_exists {
            return Ok(AuthorizationEntryResolution::ClientUnavailable);
        }

        Ok(AuthorizationEntryResolution::Candidate(
            AuthorizationEntryCandidate {
                application_id: client.application_id,
                oidc_client_id: client.id,
            },
        ))
    }

    /// Revalidates a snapshot-bound registration before a login transition.
    ///
    /// Locks and revalidates tenant -> Application -> Client -> exact redirect in the same
    /// order as [`Self::lock_authorization_entry_registration`], identifying the client by the
    /// snapshot-bound `(application_id, oidc_client_id)` pair instead of the opaque public
    /// `client_id`. The caller must retain the transaction and perform the state transition
    /// before committing it.
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_entry.lock_login_registration",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn lock_authorization_login_registration(
        tenant_id: Uuid,
        application_id: Uuid,
        oidc_client_id: Uuid,
        redirect_uri: &str,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<AuthorizationEntryLockResult, Error> {
        if Tenants::find_by_id(tenant_id)
            .filter(model::tenants::Column::Id.ne(consts::SYSTEM_TENANT_UUID))
            .lock_shared()
            .one(database)
            .await?
            .is_none()
        {
            return Ok(AuthorizationEntryLockResult::TenantUnavailable);
        }

        let Some(application) = Applications::find_by_id(application_id)
            .filter(model::applications::Column::TenantId.eq(tenant_id))
            .lock_shared()
            .one(database)
            .await?
        else {
            return Ok(AuthorizationEntryLockResult::RelationshipUnavailable);
        };

        let client_exists = OidcClients::find_by_id(oidc_client_id)
            .filter(model::oidc_clients::Column::ApplicationId.eq(application_id))
            .lock_shared()
            .one(database)
            .await?
            .is_some();
        if !client_exists {
            return Ok(AuthorizationEntryLockResult::RelationshipUnavailable);
        }

        let redirect_exists = OidcClientRedirectUris::find()
            .filter(model::oidc_client_redirect_uris::Column::OidcClientId.eq(oidc_client_id))
            .filter(model::oidc_client_redirect_uris::Column::RedirectUri.eq(redirect_uri))
            .lock_shared()
            .one(database)
            .await?
            .is_some();
        if !redirect_exists {
            return Ok(AuthorizationEntryLockResult::RelationshipUnavailable);
        }

        Ok(AuthorizationEntryLockResult::Locked(
            LockedAuthorizationEntry {
                application_id,
                oidc_client_id,
                application_configuration: application.configuration,
            },
        ))
    }

    /// Locks and revalidates tenant -> Application -> Client -> exact redirect in that order.
    /// The caller must retain the transaction and insert the snapshot before committing it.
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_authorization_entry.lock_registration",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn lock_authorization_entry_registration(
        tenant_id: Uuid,
        candidate: AuthorizationEntryCandidate,
        client_id: &str,
        redirect_uri: &str,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<AuthorizationEntryLockResult, Error> {
        let AuthorizationEntryCandidate {
            application_id,
            oidc_client_id,
        } = candidate;

        if Tenants::find_by_id(tenant_id)
            .filter(model::tenants::Column::Id.ne(consts::SYSTEM_TENANT_UUID))
            .lock_shared()
            .one(database)
            .await?
            .is_none()
        {
            return Ok(AuthorizationEntryLockResult::TenantUnavailable);
        }

        let Some(application) = Applications::find_by_id(application_id)
            .filter(model::applications::Column::TenantId.eq(tenant_id))
            .lock_shared()
            .one(database)
            .await?
        else {
            return Ok(AuthorizationEntryLockResult::RelationshipUnavailable);
        };

        let client_exists = OidcClients::find_by_id(oidc_client_id)
            .filter(model::oidc_clients::Column::ApplicationId.eq(application_id))
            .filter(model::oidc_clients::Column::ClientId.eq(client_id))
            .lock_shared()
            .one(database)
            .await?
            .is_some();
        if !client_exists {
            return Ok(AuthorizationEntryLockResult::RelationshipUnavailable);
        }

        let redirect_exists = OidcClientRedirectUris::find()
            .filter(model::oidc_client_redirect_uris::Column::OidcClientId.eq(oidc_client_id))
            .filter(model::oidc_client_redirect_uris::Column::RedirectUri.eq(redirect_uri))
            .lock_shared()
            .one(database)
            .await?
            .is_some();
        if !redirect_exists {
            return Ok(AuthorizationEntryLockResult::RelationshipUnavailable);
        }

        Ok(AuthorizationEntryLockResult::Locked(
            LockedAuthorizationEntry {
                application_id,
                oidc_client_id,
                application_configuration: application.configuration,
            },
        ))
    }
}

impl OidcAuthorizationEntryHelper for OidcClients {}
