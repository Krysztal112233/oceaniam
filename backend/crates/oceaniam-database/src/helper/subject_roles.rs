use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::SafeTransactionConnectionTrait,
    model::prelude::SubjectRoles,
    model::subject_roles::{ActiveModel, Column},
};

#[async_trait::async_trait]
pub trait SubjectRolesHelper {
    async fn get_subject_role_ids(
        subject_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<Uuid>, Error> {
        let rows = SubjectRoles::find()
            .filter(Column::SubjectId.eq(subject_id))
            .all(database)
            .await?;

        Ok(rows.into_iter().map(|r| r.role_id).collect())
    }

    async fn assign_role(
        subject_id: Uuid,
        role_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        let exists = SubjectRoles::find()
            .filter(Column::SubjectId.eq(subject_id))
            .filter(Column::RoleId.eq(role_id))
            .one(database)
            .await?
            .is_some();

        if !exists {
            SubjectRoles::insert(ActiveModel {
                subject_id: sea_orm::ActiveValue::Set(subject_id),
                role_id: sea_orm::ActiveValue::Set(role_id),
            })
            .exec(database)
            .await?;
        }

        Ok(())
    }

    async fn unassign_role(
        subject_id: Uuid,
        role_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        SubjectRoles::delete_many()
            .filter(Column::SubjectId.eq(subject_id))
            .filter(Column::RoleId.eq(role_id))
            .exec(database)
            .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl SubjectRolesHelper for SubjectRoles {}
