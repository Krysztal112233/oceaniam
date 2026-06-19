use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
    pub id: String,
    pub audit_type: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ts_rs::TS)]
#[ts(rename_all = "snake_case")]
pub struct TrendDataPoint {
    pub bucket: DateTime<FixedOffset>,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ts_rs::TS)]
#[ts(rename_all = "snake_case")]
pub struct PlatformTrendsVO {
    pub granularity: Granularity,
    pub range: u64,
    pub tenants: Vec<TrendDataPoint>,
    pub applications: Vec<TrendDataPoint>,
    pub users: Vec<TrendDataPoint>,
    pub administrators: Vec<TrendDataPoint>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ts_rs::TS)]
#[ts(rename_all = "snake_case")]
pub struct ApplicationTrendsVO {
    pub granularity: Granularity,
    pub range: u64,
    pub new_users: Vec<TrendDataPoint>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, ts_rs::TS, strum::Display)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Granularity {
    #[default]
    Day,
    Week,
    Month,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TrendQuery {
    #[serde(default)]
    pub granularity: Granularity,

    #[serde(default = "default_range")]
    pub range: u64,
}

fn default_range() -> u64 {
    30
}

impl Default for TrendQuery {
    fn default() -> Self {
        Self {
            granularity: Granularity::default(),
            range: default_range(),
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
