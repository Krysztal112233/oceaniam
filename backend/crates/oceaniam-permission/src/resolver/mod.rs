use std::collections::HashSet;

use async_trait::async_trait;
use uuid::Uuid;

use crate::permission::Permission;

pub mod builtin;

#[async_trait]
pub trait PermissionResolver: Send + Sync + std::fmt::Debug {
    async fn platform_permissions(
        &self,
        platform_id: Uuid,
    ) -> Result<HashSet<Permission>, crate::Error>;

    async fn subject_permissions(
        &self,
        subject_id: Uuid,
        application_id: Uuid,
    ) -> Result<HashSet<Permission>, crate::Error>;
}
