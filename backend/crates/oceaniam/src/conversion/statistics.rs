use oceaniam_database::{
    helper::statistics::ApplicationCounts, helper::statistics::PlatformCounts, model,
};
use oceaniam_vo::statistics::{ApplicationStatisticsVO, AuditLogVO, OverviewVO};

use super::sqid::uuid_to_sqid;

pub fn platform_counts_to_overview(counts: PlatformCounts) -> OverviewVO {
    OverviewVO {
        total_tenants: counts.total_tenants,
        total_applications: counts.total_applications,
        total_administrators: counts.total_administrators,
        total_application_users: counts.total_application_users,
        total_active_secrets: counts.total_active_secrets,
    }
}

pub fn application_counts_to_statistics(counts: ApplicationCounts) -> ApplicationStatisticsVO {
    ApplicationStatisticsVO {
        total_users: counts.total_users,
        total_active_keys: counts.total_active_keys,
    }
}

pub fn audit_log_model_to_vo(model: model::audits::Model) -> AuditLogVO {
    AuditLogVO {
        id: uuid_to_sqid(model.id),
        audit_type: model.audit_type.to_string(),
        payload: model.payload,
        created_at: model.created_at,
    }
}
