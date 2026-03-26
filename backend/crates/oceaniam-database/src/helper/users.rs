use axum::http::StatusCode;
use oceaniam_common::{PageParam, PagedResponse, error::Error};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter,
};
use uuid::Uuid;

use crate::{
    helper::{
        PagedExecutor, PagedSelect, SafeTransactionConnectionTrait, subjects::SubjectsHelper,
    },
    model::{
        self,
        prelude::{Applications, Subjects, Users},
        sea_orm_active_enums::SubjectTypeEnum,
    },
};

#[derive(Debug)]
pub struct CreateUserOpts {
    pub nickname: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug)]
pub struct CreateUserResult {
    pub user: model::users::Model,
    pub subject: model::subjects::Model,
}

#[async_trait::async_trait]
pub trait UserHelper {
    async fn create_user(
        id: Uuid,
        application_id: Uuid,
        CreateUserOpts {
            nickname,
            email,
            phone,
        }: CreateUserOpts,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<CreateUserResult, Error> {
        let user = model::users::Model {
            id,
            application_id,
            email,
            phone,
            nickname,
        }
        .into_active_model()
        .insert(database)
        .await?;

        let subject =
            Subjects::create_subjects(user.id, application_id, SubjectTypeEnum::User, database)
                .await?;

        Ok(CreateUserResult { user, subject })
    }

    async fn get_users(
        application_id: Uuid,
        page: impl Into<PageParam> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::users::Model>, Error> {
        use crate::model::users::Column::*;

        let page = page.into();

        Users::find()
            .filter(ApplicationId.eq(application_id))
            .paged(page)
            .paginate(database, page.per_page)
            .fetch_paged(page)
            .await
    }

    async fn get_all_users_of_application(
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::users::Model>, Error> {
        use crate::model::users::Column::*;

        Ok(Users::find()
            .filter(ApplicationId.eq(application_id))
            .all(database)
            .await?)
    }

    async fn get_user_of_application(
        application_id: Uuid,
        user_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::users::Model, Error> {
        use model::users::Column::*;

        Users::find()
            .filter(
                Condition::all()
                    .add(Id.eq(user_id))
                    .add(ApplicationId.eq(application_id)),
            )
            .one(database)
            .await?
            .ok_or_else(|| {
                Error::with_code(
                    StatusCode::NOT_FOUND,
                    format!("user_id={user_id} not found under application_id={application_id}"),
                )
            })
    }

    async fn get_all_users_of_tenant(
        tenant_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::users::Model>, Error> {
        use crate::model::applications::Column::*;

        Ok(Users::find()
            .find_also_related(Applications)
            .filter(TenantId.eq(tenant_id))
            .all(database)
            .await?
            .into_iter()
            .map(|(user, _)| user)
            .collect())
    }

    async fn find_by_email(
        application_id: Uuid,
        email: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::users::Model, Error> {
        use model::users::Column::*;

        match Users::find()
            .filter(
                Condition::all()
                    .add(Email.eq(email.into()))
                    .add(ApplicationId.eq(application_id)),
            )
            .one(database)
            .await
        {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(Error::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
            )),
            Err(e) => Err(Error::Db(e)),
        }
    }

    async fn find_by_phone(
        application_id: Uuid,
        phone: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::users::Model, Error> {
        use model::users::Column::*;

        match Users::find()
            .filter(
                Condition::all()
                    .add(Phone.eq(phone.into()))
                    .add(ApplicationId.eq(application_id)),
            )
            .one(database)
            .await
        {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(Error::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
            )),
            Err(e) => Err(Error::Db(e)),
        }
    }
}

impl UserHelper for Users {}
