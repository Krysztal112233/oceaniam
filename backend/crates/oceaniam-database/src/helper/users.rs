use crate::error::Error;
use axum::http::StatusCode;
use oceaniam_vo::pagination::{PageInfo, PageParam, PagedResponse};
use sea_orm::TransactionSession as _;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, Condition, EntityTrait, IntoActiveModel, Iterable, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
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
        users::Model as UserModel,
    },
};

#[derive(Debug)]
pub struct CreateUserOpts {
    pub nickname: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug)]
pub struct PatchUserOpts {
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug)]
pub struct CreateUserResult {
    pub user: UserModel,
    pub subject: model::subjects::Model,
}

#[derive(Debug, Default)]
pub struct UserContactOpts {
    pub email: Option<String>,
    pub phone: Option<String>,
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
    #[tracing::instrument(
        level = "info",
        name = "db.users.create_user",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

        let user = UserModel {
            id: subject.id,
            application_id,
            email,
            phone,
            nickname,
            created_at: chrono::Utc::now().into(),
        }
        .into_active_model()
        .insert(database)
        .await?;

        Ok(CreateUserResult { user, subject })
    }

    #[tracing::instrument(
        level = "info",
        name = "db.users.get_users",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_users(
        application_id: Uuid,
        page: Option<PageParam>,
        sort_desc: bool,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<UserModel>, Error> {
        use crate::model::{subjects::Column::ApplicationId, users};

        let base = Self::order_users_by_created_at(
            Subjects::find()
                .reverse_join(Users)
                .select_only()
                .columns(users::Column::iter())
                .filter(ApplicationId.eq(application_id)),
            sort_desc,
        )
        .into_model::<UserModel>();

        let Some(page) = page else {
            return base
                .all(database)
                .await
                .map(PagedResponse::with_entire)
                .map_err(Into::into);
        };

        let paginator = base.paginate(database, page.per_page);
        let users = paginator.fetch_page(page.page.saturating_sub(1)).await?;
        let total = paginator.num_items().await? as usize;
        let has_next = (page.as_offset() + users.len() as u64) < total as u64;

        Ok(PagedResponse {
            items: users,
            page_info: PageInfo { has_next, total },
        })
    }

    #[tracing::instrument(
        level = "info",
        name = "db.users.get_user_of_application",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_user_of_application(
        application_id: Uuid,
        user_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<UserModel, Error> {
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

    #[tracing::instrument(
        level = "info",
        name = "db.users.get_users_of_tenant",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_users_of_tenant(
        tenant_id: Uuid,
        page: Option<PageParam>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<UserModel>, Error> {
        use crate::model::applications::Column::*;

        let base = || {
            Users::find()
                .find_also_related(Applications)
                .filter(TenantId.eq(tenant_id))
        };

        let Some(page) = page else {
            return base()
                .all(database)
                .await
                .map(|rows| {
                    PagedResponse::with_entire(
                        rows.into_iter().map(|(user, _)| user).collect::<Vec<_>>(),
                    )
                })
                .map_err(Into::into);
        };

        let paginator = base().paginate(database, page.per_page);
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
    #[tracing::instrument(
        level = "info",
        name = "db.users.search_user",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn search_user(
        application_id: Uuid,
        by_nickname: Option<String>,
        by_email: Option<String>,
        by_phone: Option<String>,
        page: impl Into<PageParam> + Send,
        sort_desc: bool,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<UserModel>, Error> {
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

    #[tracing::instrument(
        level = "info",
        name = "db.users.find_contact_user",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn find_contact_user(
        application_id: Uuid,
        opts: UserContactOpts,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<UserModel, Error> {
        use model::users::Column::*;

        let condition = Condition::all()
            .add(ApplicationId.eq(application_id))
            .pipe(|it| match opts.email {
                Some(email) => it.add(Email.eq(email)),
                None => it,
            })
            .pipe(|it| match opts.phone {
                Some(phone) => it.add(Phone.eq(phone)),
                None => it,
            });

        match Users::find().filter(condition).one(database).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(Error::with_code(
                StatusCode::UNAUTHORIZED,
                oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
            )),
            Err(e) => Err(Error::Db {
                source: e,
                location: snafu::location!(),
            }),
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
                .order_by_desc(crate::model::subjects::Column::Id)
        } else {
            select
                .order_by_asc(CreatedAt)
                .order_by_asc(crate::model::subjects::Column::Id)
        }
    }

    #[tracing::instrument(
        level = "info",
        name = "db.users.patch_user",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn patch_user(
        application_id: Uuid,
        user_id: Uuid,
        patched: PatchUserOpts,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<UserModel, Error> {
        let PatchUserOpts {
            nickname,
            email,
            phone,
        } = patched;

        let tx = database.begin().await?;

        let user = {
            let mut user = Self::get_user_of_application(application_id, user_id, &tx)
                .await?
                .into_active_model();

            if let Some(nickname) = nickname {
                user.nickname = Set(nickname);
            }
            if let Some(email) = email {
                user.email = Set(Some(email));
            }
            if let Some(phone) = phone {
                user.phone = Set(Some(phone));
            }

            user
        };

        let user = user.update(&tx).await?;
        tx.commit().await?;

        Ok(user)
    }

    /// Deletes a `users` row by its ID.
    ///
    /// Ordering matters: `users.id -> subjects.id` is `ON DELETE NO ACTION`, so the `users`
    /// row must be removed *before* the matching `credentials` row (whose cascade removes
    /// the `subjects` and `subject_roles` rows). The caller is responsible for deleting the
    /// credential within the same transaction.
    #[tracing::instrument(
        level = "info",
        name = "db.users.delete_user",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn delete_user(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        Users::delete_by_id(id).exec(database).await?;
        Ok(())
    }
}

impl UserHelper for Users {}
