use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::model::sea_orm_active_enums::AuditType;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, DeriveEntityModel)]
#[sea_orm(table_name = "audit_summary_by_application")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub application_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub day: Date,
    #[sea_orm(primary_key, auto_increment = false)]
    pub audit_type: AuditType,
    pub event_count: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
