use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    error::Error, helper::SafeTransactionConnectionTrait, model::prelude::AdministratorTenants,
};

#[async_trait::async_trait]
pub trait AdministratorTenantsHelper {
    /// Probes whether an administrator holds an explicit `administrator_tenants` assignment
    /// for the given tenant.
    ///
    /// Used by the platform permission middleware to enforce the tenant organization
    /// boundary for tenant-scoped administrators.
    #[tracing::instrument(
        level = "info",
        name = "db.administrator_tenants.assignment_exists",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn assignment_exists(
        administrator_id: Uuid,
        tenant_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        use crate::model::administrator_tenants::Column::*;

        Ok(AdministratorTenants::find()
            .filter(AdministratorId.eq(administrator_id))
            .filter(TenantId.eq(tenant_id))
            .one(database)
            .await?
            .is_some())
    }
}

impl AdministratorTenantsHelper for AdministratorTenants {}
