use std::collections::HashMap;

use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::SafeTransactionConnectionTrait,
    model::prelude::RolePermissions,
    model::role_permissions::{ActiveModel, Column},
};

#[async_trait::async_trait]
pub trait RolePermissionsHelper {
    async fn get_role_permissions_map(
        role_ids: &[Uuid],
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<HashMap<Uuid, Vec<String>>, Error> {
        let rows = RolePermissions::find()
            .filter(Column::RoleId.is_in(role_ids.iter().copied()))
            .all(database)
            .await?;

        let mut map: HashMap<Uuid, Vec<String>> =
            rows.into_iter().fold(HashMap::new(), |mut acc, r| {
                acc.entry(r.role_id).or_default().push(r.permission);
                acc
            });

        for &id in role_ids {
            map.entry(id).or_default();
        }

        Ok(map)
    }

    async fn set_role_permissions(
        role_id: Uuid,
        permissions: &[String],
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        RolePermissions::delete_many()
            .filter(Column::RoleId.eq(role_id))
            .exec(database)
            .await?;

        if !permissions.is_empty() {
            let models: Vec<ActiveModel> = permissions
                .iter()
                .map(|p| ActiveModel {
                    role_id: ActiveValue::Set(role_id),
                    permission: ActiveValue::Set(p.clone()),
                })
                .collect();

            RolePermissions::insert_many(models).exec(database).await?;
        }

        Ok(())
    }
}

impl RolePermissionsHelper for RolePermissions {}
