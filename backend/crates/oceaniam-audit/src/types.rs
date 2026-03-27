use oceaniam_database::model::sea_orm_active_enums::AuditType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tagged audit payload union.
///
/// `kind` keeps the stored JSON self-describing, so it can be decoded
/// correctly from `jsonb` without depending on outside context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum AuditPayload {
    SignJwt(SignJwtPayload),
    RevokeJwt(RevokeJwtPayload),
    RefreshJwt(RefreshJwtPayload),
    CreateApplication(CreateApplicationPayload),
    PatchApplication(PatchApplicationPayload),
    PatchApplicationConfiguration(PatchApplicationConfigurationPayload),
    DeleteApplication(DeleteApplicationPayload),
    CreateTenants(CreateTenantsPayload),
    DeleteTenants(DeleteTenantsPayload),
    PatchTenant(PatchTenantPayload),
    CreateAdministrator(CreateAdministratorPayload),
    PatchAdministrator(PatchAdministratorPayload),
    CreateApplicationUser(CreateApplicationUserPayload),
    DeleteApplicationUser(DeleteApplicationUserPayload),
    CreateApplicationSecret(CreateApplicationSecretPayload),
    DeleteApplicationSecret(DeleteApplicationSecretPayload),
}

impl AuditPayload {
    pub const fn audit_type(&self) -> AuditType {
        match self {
            Self::SignJwt(_) => AuditType::SignJwt,
            Self::RevokeJwt(_) => AuditType::RevokeJwt,
            Self::RefreshJwt(_) => AuditType::RefreshJwt,
            Self::CreateApplication(_) => AuditType::CreateApplication,
            Self::PatchApplication(_) => AuditType::PatchApplication,
            Self::PatchApplicationConfiguration(_) => AuditType::PatchApplicationConfiguration,
            Self::DeleteApplication(_) => AuditType::DeleteApplication,
            Self::CreateTenants(_) => AuditType::CreateTenants,
            Self::DeleteTenants(_) => AuditType::DeleteTenants,
            Self::PatchTenant(_) => AuditType::PatchTenant,
            Self::CreateAdministrator(_) => AuditType::CreateAdministrator,
            Self::PatchAdministrator(_) => AuditType::PatchAdministrator,
            Self::CreateApplicationUser(_) => AuditType::CreateApplicationUser,
            Self::DeleteApplicationUser(_) => AuditType::DeleteApplicationUser,
            Self::CreateApplicationSecret(_) => AuditType::CreateApplicationSecret,
            Self::DeleteApplicationSecret(_) => AuditType::DeleteApplicationSecret,
        }
    }

    pub fn into_json(self) -> serde_json::Result<serde_json::Value> {
        serde_json::to_value(self)
    }
}

impl From<SignJwtPayload> for AuditPayload {
    fn from(value: SignJwtPayload) -> Self {
        Self::SignJwt(value)
    }
}

impl From<RevokeJwtPayload> for AuditPayload {
    fn from(value: RevokeJwtPayload) -> Self {
        Self::RevokeJwt(value)
    }
}

impl From<RefreshJwtPayload> for AuditPayload {
    fn from(value: RefreshJwtPayload) -> Self {
        Self::RefreshJwt(value)
    }
}

impl From<CreateApplicationPayload> for AuditPayload {
    fn from(value: CreateApplicationPayload) -> Self {
        Self::CreateApplication(value)
    }
}

impl From<PatchApplicationPayload> for AuditPayload {
    fn from(value: PatchApplicationPayload) -> Self {
        Self::PatchApplication(value)
    }
}

impl From<PatchApplicationConfigurationPayload> for AuditPayload {
    fn from(value: PatchApplicationConfigurationPayload) -> Self {
        Self::PatchApplicationConfiguration(value)
    }
}

impl From<DeleteApplicationPayload> for AuditPayload {
    fn from(value: DeleteApplicationPayload) -> Self {
        Self::DeleteApplication(value)
    }
}

impl From<CreateTenantsPayload> for AuditPayload {
    fn from(value: CreateTenantsPayload) -> Self {
        Self::CreateTenants(value)
    }
}

impl From<DeleteTenantsPayload> for AuditPayload {
    fn from(value: DeleteTenantsPayload) -> Self {
        Self::DeleteTenants(value)
    }
}

impl From<PatchTenantPayload> for AuditPayload {
    fn from(value: PatchTenantPayload) -> Self {
        Self::PatchTenant(value)
    }
}

impl From<CreateAdministratorPayload> for AuditPayload {
    fn from(value: CreateAdministratorPayload) -> Self {
        Self::CreateAdministrator(value)
    }
}

impl From<PatchAdministratorPayload> for AuditPayload {
    fn from(value: PatchAdministratorPayload) -> Self {
        Self::PatchAdministrator(value)
    }
}

impl From<CreateApplicationUserPayload> for AuditPayload {
    fn from(value: CreateApplicationUserPayload) -> Self {
        Self::CreateApplicationUser(value)
    }
}

impl From<DeleteApplicationUserPayload> for AuditPayload {
    fn from(value: DeleteApplicationUserPayload) -> Self {
        Self::DeleteApplicationUser(value)
    }
}

impl From<CreateApplicationSecretPayload> for AuditPayload {
    fn from(value: CreateApplicationSecretPayload) -> Self {
        Self::CreateApplicationSecret(value)
    }
}

impl From<DeleteApplicationSecretPayload> for AuditPayload {
    fn from(value: DeleteApplicationSecretPayload) -> Self {
        Self::DeleteApplicationSecret(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct SignJwtPayload {
    pub application_id: Uuid,
    pub subject_id: Uuid,
    pub jti: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct RevokeJwtPayload {
    pub subject_id: Uuid,
    pub jti: Uuid,
    pub application_id: Option<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct RefreshJwtPayload {
    pub application_id: Uuid,
    pub subject_id: Uuid,
    pub old_jti: Uuid,
    pub new_jti: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct CreateApplicationPayload {
    pub application_id: Uuid,
    pub tenant_id: Uuid,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct PatchApplicationPayload {
    pub application_id: Uuid,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct PatchApplicationConfigurationPayload {
    pub application_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct DeleteApplicationPayload {
    pub application_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct CreateTenantsPayload {
    pub tenant_id: Uuid,
    pub comment: Option<String>,
    pub operator_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct DeleteTenantsPayload {
    pub tenant_id: Uuid,
    pub operator_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct PatchTenantPayload {
    pub tenant_id: Uuid,
    pub operator_id: Uuid,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct CreateAdministratorPayload {
    pub administrator_id: Uuid,
    pub operator_id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct PatchAdministratorPayload {
    pub target_id: Uuid,
    pub operator_id: Uuid,
    pub name: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct CreateApplicationUserPayload {
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nickname: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct DeleteApplicationUserPayload {
    pub application_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct CreateApplicationSecretPayload {
    pub operator_id: Uuid,
    pub secret_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct DeleteApplicationSecretPayload {
    pub operator_id: Uuid,
    pub secret_id: Uuid,
}
