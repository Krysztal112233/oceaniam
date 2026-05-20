use std::collections::HashMap;

use oceaniam_api::PagedResponse;
use oceaniam_vo::statistics::*;
use reqwest::Method;

use crate::client::{AuthMode, OceanIamClient};
use crate::error::Error;
use crate::paths;

impl OceanIamClient {
    pub async fn get_statistics(&self) -> Result<OverviewVO, Error> {
        let req = self.auth_req(Method::GET, paths::STATISTICS, AuthMode::Bearer)?;
        self.send_inner(req).await
    }

    pub async fn get_application_statistics(
        &self,
        tenant_id: &str,
        application_id: &str,
    ) -> Result<ApplicationStatisticsVO, Error> {
        let path = paths::fmt2(paths::APP_STATISTICS, tenant_id, application_id);
        let req = self.auth_req(Method::GET, &path, AuthMode::BearerOrAppSecret)?;
        self.send_inner(req).await
    }

    pub async fn get_application_audits(
        &self,
        tenant_id: &str,
        application_id: &str,
        page: Option<u64>,
        per_page: Option<u64>,
        audit_type: Option<&str>,
    ) -> Result<PagedResponse<AuditLogVO>, Error> {
        let path = paths::fmt2(paths::APP_AUDITS, tenant_id, application_id);
        let mut req = self.auth_req(Method::GET, &path, AuthMode::BearerOrAppSecret)?;
        let mut params = HashMap::new();
        if let Some(v) = page {
            params.insert("page", v.to_string());
        }
        if let Some(v) = per_page {
            params.insert("per_page", v.to_string());
        }
        if let Some(v) = audit_type {
            params.insert("audit_type", v.to_string());
        }
        if !params.is_empty() {
            req = req.query(&params);
        }
        self.send_inner(req).await
    }

    pub async fn get_audit_logs(
        &self,
        page: Option<u64>,
        per_page: Option<u64>,
        audit_type: Option<&str>,
    ) -> Result<PagedResponse<AuditLogVO>, Error> {
        let mut req = self.auth_req(Method::GET, paths::AUDITS, AuthMode::Bearer)?;
        let mut params = HashMap::new();
        if let Some(v) = page {
            params.insert("page", v.to_string());
        }
        if let Some(v) = per_page {
            params.insert("per_page", v.to_string());
        }
        if let Some(v) = audit_type {
            params.insert("audit_type", v.to_string());
        }
        if !params.is_empty() {
            req = req.query(&params);
        }
        self.send_inner(req).await
    }
}
