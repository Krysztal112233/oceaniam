use axum::http::StatusCode;
use oceaniam_common::{PageInfo, PageParam, PagedResponse, error::Error};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QueryOrder,
    sea_query::{Expr, extension::postgres::PgExpr},
};
use tap::Pipe;
use uuid::Uuid;

use crate::{
    helper::{SafeTransactionConnectionTrait, subjects::SubjectsHelper},
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
    /// Creates the `subjects` and `users` rows for an application user.
    ///
    /// Ordering matters here. The caller must create the matching credential
    /// row first, using the same UUID, before calling this helper.
    ///
    /// That requirement exists because `subjects.id` is linked to
    /// `credentials.id` by a foreign key in the current schema. This helper
    /// therefore only creates the subject and user records, and assumes the
    /// credential already exists inside the same transaction when atomicity is
    /// required.
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
        let subject =
            Subjects::create_subjects(id, application_id, SubjectTypeEnum::User, database).await?;

        let user = model::users::Model {
            id: subject.id,
            application_id,
            email,
            phone,
            nickname,
        }
        .into_active_model()
        .insert(database)
        .await?;

        Ok(CreateUserResult { user, subject })
    }

    async fn get_users(
        application_id: Uuid,
        page: impl Into<PageParam> + Send,
        sort_desc: bool,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::users::Model>, Error> {
        use crate::model::users::Column::ApplicationId;

        let page = page.into();
        let paginator = Self::order_users_by_created_at(
            Users::find()
                .inner_join(Subjects)
                .filter(ApplicationId.eq(application_id)),
            sort_desc,
        )
        .paginate(database, page.per_page);
        let users = paginator.fetch_page(page.page.saturating_sub(1)).await?;
        let total = paginator.num_items().await? as usize;
        let has_next = (page.as_offset() + users.len() as u64) < total as u64;

        Ok(PagedResponse {
            items: users,
            page_info: PageInfo { has_next, total },
        })
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

    async fn get_users_of_tenant(
        tenant_id: Uuid,
        page: impl Into<PageParam> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::users::Model>, Error> {
        use crate::model::applications::Column::*;

        let page = page.into();
        let paginator = Users::find()
            .find_also_related(Applications)
            .filter(TenantId.eq(tenant_id))
            .paginate(database, page.per_page);
        let users = paginator
            .fetch_page(page.page.saturating_sub(1))
            .await?
            .into_iter()
            .map(|(user, _)| user)
            .collect::<Vec<_>>();
        let total = paginator.num_items().await? as usize;
        let has_next = (page.as_offset() + users.len() as u64) < total as u64;

        Ok(PagedResponse {
            items: users,
            page_info: PageInfo { has_next, total },
        })
    }

    /// NOTE: Caller must escape user input for LIKE/ILIKE wildcard semantics before calling.
    async fn search_user(
        application_id: Uuid,
        by_nickname: Option<String>,
        by_email: Option<String>,
        by_phone: Option<String>,
        page: impl Into<PageParam> + Send,
        sort_desc: bool,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::users::Model>, Error> {
        use crate::model::users::Column::{ApplicationId, Email, Nickname, Phone};

        let page = page.into();

        let by_nickname = by_nickname
            .as_deref()
            .map(str::trim)
            .filter(|it| !it.is_empty());

        let by_email = by_email
            .as_deref()
            .map(str::trim)
            .filter(|it| !it.is_empty());

        let by_phone = by_phone
            .as_deref()
            .map(str::trim)
            .filter(|it| !it.is_empty());

        let condition = Condition::all()
            .add(ApplicationId.eq(application_id))
            .pipe(|it| match by_nickname {
                Some(by_nickname) => it.add(Expr::col(Nickname).ilike(format!("%{by_nickname}%"))),
                _ => it,
            })
            .pipe(|it| match by_email {
                Some(by_email) => it.add(Expr::col(Email).ilike(format!("%{by_email}%"))),
                _ => it,
            })
            .pipe(|it| match by_phone {
                Some(by_phone) => it.add(Expr::col(Phone).ilike(format!("%{by_phone}%"))),
                _ => it,
            });

        let paginator = Self::order_users_by_created_at(
            Users::find().inner_join(Subjects).filter(condition),
            sort_desc,
        )
        .paginate(database, page.per_page);
        let users = paginator.fetch_page(page.page.saturating_sub(1)).await?;
        let total = paginator.num_items().await? as usize;
        let has_next = (page.as_offset() + users.len() as u64) < total as u64;

        Ok(PagedResponse {
            items: users,
            page_info: PageInfo { has_next, total },
        })
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

    fn order_users_by_created_at<S>(select: S, sort_desc: bool) -> S
    where
        S: QueryOrder,
    {
        use crate::model::subjects::Column::CreatedAt;

        if sort_desc {
            select
                .order_by_desc(CreatedAt)
                .order_by_desc(crate::model::users::Column::Id)
        } else {
            select
                .order_by_asc(CreatedAt)
                .order_by_asc(crate::model::users::Column::Id)
        }
    }
}

impl UserHelper for Users {}
