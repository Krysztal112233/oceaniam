use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use moka::future::Cache;
use snafu;
use tracing::instrument;
use uuid::Uuid;

use super::PermissionResolver;
use crate::permission::Permission;
use crate::role::PlatformRole;

use oceaniam_common::consts;
use oceaniam_database::model::prelude::*;
use sea_orm::{DatabaseConnection, EntityTrait};

#[derive(Hash, Eq, PartialEq, Clone)]
struct SubjectCacheKey {
    subject_id: Uuid,
    application_id: Uuid,
}

pub struct BuiltinResolver {
    db: DatabaseConnection,
    platform_cache: Cache<Uuid, HashSet<Permission>>,
    subject_cache: Cache<SubjectCacheKey, HashSet<Permission>>,
}

impl BuiltinResolver {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            platform_cache: Cache::builder()
                .max_capacity(256)
                .time_to_live(Duration::from_secs(10))
                .build(),
            subject_cache: Cache::builder()
                .max_capacity(2048)
                .time_to_live(Duration::from_secs(10))
                .build(),
        }
    }
}

#[async_trait]
impl PermissionResolver for BuiltinResolver {
    #[instrument(skip(self), fields(%platform_id))]
    async fn platform_permissions(
        &self,
        platform_id: Uuid,
    ) -> Result<HashSet<Permission>, crate::Error> {
        let result = self
            .platform_cache
            .try_get_with(platform_id, async {
                resolve_platform_permissions(&self.db, platform_id).await
            })
            .await
            .map_err(|arc| match Arc::try_unwrap(arc) {
                Ok(e) => e,
                Err(arc) => crate::Error::Internal {
                    msg: format!("cache error: {arc}"),
                    location: snafu::location!(),
                },
            })?;
        Ok(result)
    }

    #[instrument(skip(self), fields(%subject_id, %application_id))]
    async fn subject_permissions(
        &self,
        subject_id: Uuid,
        application_id: Uuid,
    ) -> Result<HashSet<Permission>, crate::Error> {
        let result = self
            .subject_cache
            .try_get_with(
                SubjectCacheKey {
                    subject_id,
                    application_id,
                },
                async { resolve_subject_permissions(&self.db, subject_id, application_id).await },
            )
            .await
            .map_err(|arc| match Arc::try_unwrap(arc) {
                Ok(e) => e,
                Err(arc) => crate::Error::Internal {
                    msg: format!("cache error: {arc}"),
                    location: snafu::location!(),
                },
            })?;
        Ok(result)
    }
}

/// Read-through callback invoked on `platform_cache` miss.
///
/// Returns the permission set for a given platform administrator:
/// - The system built-in administrator always gets [`PlatformRole::SuperAdmin`].
/// - Real administrators are looked up in the `administrators` table,
///   and their `role` column is parsed into a [`PlatformRole`].
async fn resolve_platform_permissions(
    db: &DatabaseConnection,

    platform_id: Uuid,
) -> Result<HashSet<Permission>, crate::Error> {
    if platform_id == consts::SYSTEM_APPLICATION_UUID {
        return Ok(PlatformRole::SuperAdmin.permissions().clone());
    }

    let admin = Administrators::find_by_id(platform_id)
        .one(db)
        .await
        .map_err(|e| crate::Error::DatabaseRaw {
            source: e,
            location: snafu::location!(),
        })?
        .ok_or_else(|| crate::Error::Internal {
            msg: "administrator not found".to_string(),
            location: snafu::location!(),
        })?;

    match admin.role.as_deref() {
        Some("super_admin") => Ok(PlatformRole::SuperAdmin.permissions().clone()),
        Some("tenant_admin") => Ok(PlatformRole::TenantAdmin.permissions().clone()),
        Some("readonly_admin") => Ok(PlatformRole::ReadonlyAdmin.permissions().clone()),
        Some(other) => Err(crate::Error::Internal {
            msg: format!("unknown platform role: {other}"),
            location: snafu::location!(),
        }),
        None => Err(crate::Error::Internal {
            msg: "administrator has no role assigned".to_string(),
            location: snafu::location!(),
        }),
    }
}

async fn resolve_subject_permissions(
    _db: &DatabaseConnection,
    _subject_id: Uuid,
    _application_id: Uuid,
) -> Result<HashSet<Permission>, crate::Error> {
    // TODO: resolve from subjects.application_role_id → application_roles (Phase 5)
    Ok(HashSet::new())
}
