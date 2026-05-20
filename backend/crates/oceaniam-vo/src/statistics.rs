use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::sqid::Sqid;
#[cfg(feature = "database")]
use oceaniam_database::{
    helper::statistics::{ApplicationCounts, PlatformCounts},
    model,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, ts_rs::TS)]
#[ts(rename_all = "snake_case")]
pub struct OverviewVO {
    pub total_tenants: u64,
    pub total_applications: u64,
    pub total_administrators: u64,
    pub total_application_users: u64,
    pub total_active_secrets: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ts_rs::TS)]
#[ts(rename_all = "snake_case")]
pub struct ApplicationStatisticsVO {
    pub total_users: u64,
    pub total_active_keys: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ts_rs::TS)]
#[ts(rename_all = "snake_case")]
pub struct AuditLogVO {
    pub id: Sqid,
    pub audit_type: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<FixedOffset>,
}

#[cfg(feature = "database")]
impl From<PlatformCounts> for OverviewVO {
    fn from(pc: PlatformCounts) -> Self {
        Self {
            total_tenants: pc.total_tenants,
            total_applications: pc.total_applications,
            total_administrators: pc.total_administrators,
            total_application_users: pc.total_application_users,
            total_active_secrets: pc.total_active_secrets,
        }
    }
}

#[cfg(feature = "database")]
impl From<ApplicationCounts> for ApplicationStatisticsVO {
    fn from(ac: ApplicationCounts) -> Self {
        Self {
            total_users: ac.total_users,
            total_active_keys: ac.total_active_keys,
        }
    }
}

#[cfg(feature = "database")]
impl From<model::audits::Model> for AuditLogVO {
    fn from(m: model::audits::Model) -> Self {
        Self {
            id: m.id.into(),
            audit_type: m.audit_type.to_string(),
            payload: m.payload,
            created_at: m.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuditLogQuery {
    #[serde(default)]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
    pub audit_type: Option<String>,
}

fn default_per_page() -> u64 {
    30
}

impl Default for AuditLogQuery {
    fn default() -> Self {
        Self {
            page: 0,
            per_page: 30,
            audit_type: None,
        }
    }
}
