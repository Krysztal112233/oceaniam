use std::{collections::HashMap, time::Duration};

use argon2::{Argon2, Params};
use axum::http::StatusCode;
use moka::future::Cache;
use oceaniam_api::{PageParam, PagedResponse};
use oceaniam_common::{error::Error, helpers::gen_random_with_charset};
use oceaniam_database::config::application::ApplicationConfiguration;
use oceaniam_database::helper::applications::CreateApplicationOptions;
use oceaniam_database::helper::users::{CreateUserOpts, CreateUserResult};
use oceaniam_database::{
    helper::{SafeTransactionConnectionTrait, users::UserHelper},
    model::application_secrets::Model as SecretModel,
    model::applications::Model as ApplicationModel,
};
use oceaniam_database::{
    helper::{applications::ApplicationHelper, applications_secrets::ApplicationSecretsHelper},
    model::{
        prelude::{ApplicationSecrets, Applications, Users},
        users::Model as UserModel,
    },
};
use oceaniam_vo::applications::{
    PatchApplicationConfigurationRequest, PatchApplicationRequest, PatchValue,
};
use oceaniam_vo::auth::AuthVO;
use sea_orm::prelude::*;
use tap::Tap;
use tracing::{error, info};
use uuid::Uuid;

use crate::state::credentials::ManagedCredentialVaults;
use crate::state::filters::ManagedFilters;

fn build_argon2(configuration: &ApplicationConfiguration) -> Result<Argon2<'static>, Error> {
    let params = Params::new(
        configuration.auth.password.argon2.m_cost,
        configuration.auth.password.argon2.t_cost,
        configuration.auth.password.argon2.p_cost,
        Some(Params::DEFAULT_OUTPUT_LEN),
    )
    .map_err(|error| Error::with_code(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    ))
}

/// TODO: In future, using [xorf] to detect does [Applications] existed in database for higher performance.
#[derive(Debug, Clone)]
pub struct ManagedApplications<'a> {
    database: DatabaseConnection,

    secrets: Secrets<'a>,
    users: Cache<Uuid, ApplicationUsers>,

    configurations: Cache<Uuid, ApplicationConfiguration>,

    /// This field are shared with global states.
    shared_credential_vaults: ManagedCredentialVaults,

    filters: ManagedFilters<'a>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum UserIdentifier {
    Email(String),
    Phone(String),
    Id(Uuid),
}

impl From<AuthVO> for UserIdentifier {
    fn from(value: AuthVO) -> Self {
        match value {
            AuthVO::Email { email, .. } => UserIdentifier::Email(email),
            AuthVO::Phone { phone, .. } => UserIdentifier::Phone(phone),
        }
    }
}

impl ManagedApplications<'_> {
    pub fn new<'a>(
        filters: ManagedFilters<'a>,
        credential: ManagedCredentialVaults,
        database: DatabaseConnection,
    ) -> ManagedApplications<'a> {
        ManagedApplications {
            database: database.clone(),
            users: Cache::builder()
                .time_to_idle(Duration::from_mins(30))
                .build(),
            secrets: Secrets::new(filters.clone(), database),
            configurations: Cache::builder()
                .time_to_idle(Duration::from_mins(30))
                .build(),

            shared_credential_vaults: credential,
            filters,
        }
    }

    pub async fn get_application_users(
        &self,
        application_id: Uuid,
    ) -> Result<ApplicationUsers, Error> {
        self.is_application_exist(application_id).await?;

        Ok(self
            .users
            .try_get_with(
                application_id,
                ApplicationUsers::new(
                    application_id,
                    self.shared_credential_vaults.clone(),
                    self.database.clone(),
                ),
            )
            .await?)
    }

    #[allow(private_bounds)]
    pub async fn find_user_by(
        &self,
        application_id: Uuid,
        user_identifier: impl Into<UserIdentifier> + Send,
    ) -> Result<UserModel, Error> {
        self.is_application_exist(application_id).await?;

        self.get_application_users(application_id)
            .await?
            .find_user_by(user_identifier.into())
            .await
    }

    pub async fn delete_application(&self, application_id: Uuid) -> Result<(), Error> {
        self.is_application_exist(application_id).await?;

        info!("deleting application: id={application_id}");

        Applications::delete_application(application_id, &self.database)
            .await
            .inspect_err(|e| error!("{e}"))
            .inspect(|_| info!("application deleted successfully: id={application_id}"))?;

        self.filters.application_id_filter().mark();
        self.filters.secret_filter().mark();
        self.filters.secret_id_filter().mark();

        Ok(())
    }

    pub async fn create_application(
        &self,
        tenant_id: Uuid,
        comment: Option<String>,
    ) -> Result<ApplicationModel, Error> {
        let model = Applications::create_with_opts(
            Uuid::now_v7(),
            tenant_id,
            CreateApplicationOptions {
                comment,
                ..CreateApplicationOptions::default()
            },
            &self.database,
        )
        .await
        .inspect_err(|e| error!("{e}"))?;

        self.filters.application_id_filter().mark();

        Ok(model)
    }

    pub async fn get_configuration(
        &self,
        application_id: Uuid,
    ) -> Result<ApplicationConfiguration, Error> {
        self.is_application_exist(application_id).await?;

        Ok(self
            .configurations
            .try_get_with(application_id, async {
                Ok(
                    Applications::get_application(application_id, &self.database)
                        .await?
                        .into(),
                )
            })
            .await?)
    }

    pub async fn patch_configuration(
        &self,
        application_id: Uuid,
        patch: PatchApplicationConfigurationRequest,
    ) -> Result<ApplicationConfiguration, Error> {
        self.is_application_exist(application_id).await?;

        let patched_configuration = self.get_configuration(application_id).await?.tap_mut(|it| {
            if let Some(auth) = patch.auth
                && let Some(token) = auth.token
            {
                if let Some(issuer) = token.issuer {
                    it.auth.token.issuer = issuer;
                }

                if let Some(audience) = token.audience {
                    it.auth.token.audience = audience;
                }
            }

            if let Some(registration) = patch.registration
                && let Some(enabled) = registration.enabled
            {
                it.registration.enabled = enabled;
            }
        });

        let configuration = ApplicationConfiguration::from(
            Applications::replace_configuration(
                application_id,
                patched_configuration,
                &self.database,
            )
            .await?,
        );

        self.configurations
            .insert(application_id, configuration.clone())
            .await;

        Ok(configuration)
    }

    pub async fn patch_application(
        &self,
        application_id: Uuid,
        patch: PatchApplicationRequest,
    ) -> Result<ApplicationModel, Error> {
        self.is_application_exist(application_id).await?;

        match patch.comment {
            PatchValue::Missing => {
                Applications::get_application(application_id, &self.database).await
            }
            PatchValue::Null => {
                Applications::update_comment(application_id, None, &self.database).await
            }
            PatchValue::Value(comment) => {
                Applications::update_comment(application_id, Some(comment), &self.database).await
            }
        }
    }

    async fn is_application_exist(&self, application_id: Uuid) -> Result<(), Error> {
        if self.filters.application_id_filter().exists(&application_id) {
            return Ok(());
        }

        if Applications::is_exist(application_id, &self.database).await? {
            self.filters.application_id_filter().mark();
            return Ok(());
        }

        Err(Error::with_code(
            StatusCode::NOT_FOUND,
            format!("application_id={application_id} doesn't exist"),
        ))
    }
}

impl<'a> ManagedApplications<'a> {
    pub fn secrets(&self) -> &Secrets<'a> {
        &self.secrets
    }
}

/// If [ApplicationUsers::application_id] doesn't existed in database, the entire functions of this structure will be noop
#[derive(Debug, Clone)]
pub struct ApplicationUsers {
    application_id: Uuid,
    database: DatabaseConnection,
    cache: Cache<UserIdentifier, UserModel>,

    shared_credential_vaults: ManagedCredentialVaults,
}

#[derive(Debug)]
pub struct UserSearchOptions {
    pub by_nickname: Option<String>,
    pub by_email: Option<String>,
    pub by_phone: Option<String>,
}

impl ApplicationUsers {
    async fn new(
        application_id: Uuid,
        shared_credential_vaults: ManagedCredentialVaults,
        database: DatabaseConnection,
    ) -> Result<Self, Error> {
        if Applications::is_exist(application_id, &database).await? {
            Ok(Self {
                application_id,
                database,
                cache: Cache::builder()
                    .time_to_idle(Duration::from_secs(30))
                    .build(),
                shared_credential_vaults,
            })
        } else {
            Err(Error::with_code(
                StatusCode::NOT_FOUND,
                format!("application_id={application_id} not found"),
            ))
        }
    }

    pub async fn find_user_by(&self, user_identifier: UserIdentifier) -> Result<UserModel, Error> {
        Ok(self
            .cache
            .try_get_with(user_identifier.clone(), async move {
                match user_identifier {
                    UserIdentifier::Email(mail) => {
                        Users::find_by_email(self.application_id, mail, &self.database).await
                    }
                    UserIdentifier::Phone(phone) => {
                        Users::find_by_phone(self.application_id, phone, &self.database).await
                    }
                    UserIdentifier::Id(uuid) => {
                        Users::get_user_of_application(self.application_id, uuid, &self.database)
                            .await
                    }
                }
            })
            .await?)
    }

    pub async fn create_user(
        &self,
        application_id: Uuid,
        opts: CreateUserOpts,
        password: impl Into<String>,
    ) -> Result<UserModel, Error> {
        let user = self
            .create_user_in_tx(application_id, opts, password, &self.database)
            .await?;

        Ok(user)
    }

    pub async fn create_user_in_tx(
        &self,
        application_id: Uuid,
        opts: CreateUserOpts,
        password: impl Into<String>,
        transaction: &impl SafeTransactionConnectionTrait,
    ) -> Result<UserModel, Error> {
        let user_id = Uuid::now_v7();
        let password = password.into();
        let argon2 = build_argon2(&ApplicationConfiguration::from(
            Applications::get_application(application_id, transaction).await?,
        ))?;

        info!(
            "creating new user: user_id={}, application_id={}",
            user_id, application_id
        );

        self.shared_credential_vaults
            .create_with_password_in_tx(user_id, password, &argon2, transaction)
            .await
            .inspect_err(|e| {
                error!(
                    "failed to create credential for user: user_id={}, subject_id={}, error={}",
                    user_id, user_id, e
                );
            })?;

        let CreateUserResult { user, subject } =
            Users::create_user(user_id, application_id, opts, transaction)
                .await
                .inspect_err(|e| {
                    error!(
                        "failed to create user: user_id={}, application_id={}, error={}",
                        user_id, application_id, e
                    );
                })?;

        info!(
            "user created successfully: user_id={}, subject_id={}, application_id={}",
            user.id, subject.id, application_id
        );

        Ok(user)
    }
}

/// If [ApplicationUsers::application_id] doesn't existed in database, the entire functions of this structure will be noop
///
/// TODO: In future, using [xorf] to detect does secret existed in database for higher performance.
#[derive(Debug, Clone)]
struct SecretCaches {
    by_application: Cache<Uuid, Vec<SecretModel>>,
    by_id: Cache<Uuid, SecretModel>,
    application_ids_by_secret_id: Cache<Uuid, Vec<Uuid>>,
    application_ids_by_secret: Cache<String, Vec<Uuid>>,
}

impl SecretCaches {
    fn new() -> Self {
        Self {
            by_application: Cache::builder()
                .time_to_idle(Duration::from_mins(5))
                .build(),
            by_id: Cache::builder()
                .time_to_idle(Duration::from_mins(5))
                .build(),
            application_ids_by_secret_id: Cache::builder()
                .time_to_idle(Duration::from_mins(5))
                .build(),
            application_ids_by_secret: Cache::builder()
                .time_to_idle(Duration::from_mins(5))
                .build(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Secrets<'a> {
    database: DatabaseConnection,
    caches: SecretCaches,
    filters: ManagedFilters<'a>,
}

impl Secrets<'_> {
    pub fn new<'a>(filters: ManagedFilters<'a>, database: DatabaseConnection) -> Secrets<'a> {
        Secrets {
            database,
            caches: SecretCaches::new(),
            filters,
        }
    }

    pub async fn create_secret(&self) -> Result<SecretModel, Error> {
        let model =
            ApplicationSecrets::create_secret_unbound(Uuid::now_v7(), gen_secret(), &self.database)
                .await?;

        self.cache_secret_model(&model).await;
        self.cache_secret_application_ids(model.id, Vec::new())
            .await;

        self.filters.secret_filter().mark();
        self.filters.secret_id_filter().mark();

        Ok(model)
    }

    pub async fn get_all_secrets(&self) -> Result<Vec<SecretModel>, Error> {
        let models = ApplicationSecrets::get_all_secret_models(&self.database).await?;

        for model in &models {
            self.cache_secret_model(model).await;
        }

        Ok(models)
    }

    pub async fn get_secret_models(
        &self,
        page: PageParam,
    ) -> Result<PagedResponse<SecretModel>, Error> {
        let paged = ApplicationSecrets::get_secret_models(page, &self.database).await?;

        for model in &paged.items {
            self.cache_secret_model(model).await;
        }

        Ok(paged)
    }

    pub async fn get_secret(&self, secret_id: Uuid) -> Result<SecretModel, Error> {
        Ok(self
            .caches
            .by_id
            .try_get_with(secret_id, async {
                ApplicationSecrets::get_secret(secret_id, &self.database).await
            })
            .await?)
    }

    pub async fn get_secret_application_ids(&self, secret_id: Uuid) -> Result<Vec<Uuid>, Error> {
        Ok(self
            .caches
            .application_ids_by_secret_id
            .try_get_with(secret_id, async {
                ApplicationSecrets::get_application_ids_of_secret(secret_id, &self.database).await
            })
            .await?)
    }

    pub async fn get_secret_application_ids_batch(
        &self,
    ) -> Result<HashMap<Uuid, Vec<Uuid>>, Error> {
        ApplicationSecrets::get_all_application_ids_grouped_by_secret_id(&self.database).await
    }

    pub async fn get_secret_application_ids_batch_by_ids(
        &self,
        secret_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, Vec<Uuid>>, Error> {
        let application_ids_by_secret =
            ApplicationSecrets::get_application_ids_grouped_by_secret_ids(
                secret_ids,
                &self.database,
            )
            .await?;

        for (secret_id, application_ids) in &application_ids_by_secret {
            self.cache_secret_application_ids(*secret_id, application_ids.clone())
                .await;
        }

        Ok(application_ids_by_secret)
    }

    async fn refresh(&self, application_id: Uuid) -> Result<(), Error> {
        self.is_application_exist(application_id).await?;

        self.caches
            .by_application
            .insert(
                application_id,
                ApplicationSecrets::get_all_secrets_of(application_id, &self.database).await?,
            )
            .await;

        Ok(())
    }

    /// TODO: See [Secrets]
    pub async fn find_secret_belong_to(
        &self,
        secret: impl Into<String>,
    ) -> Result<Vec<Uuid>, Error> {
        let secret = secret.into();

        self.is_secret_exist(&secret).await?;

        Ok(self
            .caches
            .application_ids_by_secret
            .try_get_with(secret.clone(), async {
                ApplicationSecrets::find_secret_can_be_used_for(secret, &self.database).await
            })
            .await?)
    }

    pub async fn get_all_secrets_of(
        &self,
        application_id: Uuid,
    ) -> Result<Vec<SecretModel>, Error> {
        self.is_application_exist(application_id).await?;

        Ok(self
            .caches
            .by_application
            .try_get_with(application_id, async {
                ApplicationSecrets::get_all_secrets_of(application_id, &self.database).await
            })
            .await?)
    }

    pub async fn delete_secret(&self, application_id: Uuid, secret_id: Uuid) -> Result<(), Error> {
        self.is_application_exist(application_id).await?;
        self.is_secret_id_exist(secret_id).await?;

        ApplicationSecrets::delete_secret(application_id, secret_id, &self.database).await?;

        self.refresh(application_id).await?;
        self.invalidate_secret_bindings(secret_id).await;

        self.filters.secret_id_filter().mark();
        self.filters.secret_filter().mark();

        Ok(())
    }

    pub async fn delete_secret_by_id(&self, secret_id: Uuid) -> Result<(), Error> {
        self.is_secret_id_exist(secret_id).await?;

        let application_ids =
            ApplicationSecrets::get_application_ids_of_secret(secret_id, &self.database).await?;

        ApplicationSecrets::delete_secret_by_id(secret_id, &self.database).await?;

        for application_id in application_ids {
            self.invalidate_application_secrets(application_id).await;
        }

        self.invalidate_secret(secret_id).await;

        self.filters.secret_id_filter().mark();
        self.filters.secret_filter().mark();

        Ok(())
    }

    async fn is_secret_id_exist(&self, secret_id: Uuid) -> Result<(), Error> {
        if self.filters.secret_id_filter().exists(&secret_id) {
            Ok(())
        } else {
            Err(Error::with_code(
                StatusCode::NOT_FOUND,
                format!("secret_id={secret_id} doesn't exist"),
            ))
        }
    }

    async fn is_secret_exist(&self, secret: impl Into<String>) -> Result<(), Error> {
        let secret = secret.into();

        if self.filters.secret_filter().exists(&secret) {
            Ok(())
        } else {
            Err(Error::with_code(
                StatusCode::NOT_FOUND,
                format!("secret={} doesn't exist", "*".repeat(secret.len())),
            ))
        }
    }

    async fn is_application_exist(&self, application_id: Uuid) -> Result<(), Error> {
        if self.filters.application_id_filter().exists(&application_id) {
            Ok(())
        } else {
            Err(Error::with_code(
                StatusCode::NOT_FOUND,
                format!("application_id={application_id} doesn't exist"),
            ))
        }
    }

    async fn cache_secret_model(&self, model: &SecretModel) {
        self.caches.by_id.insert(model.id, model.clone()).await;
    }

    async fn cache_secret_application_ids(&self, secret_id: Uuid, application_ids: Vec<Uuid>) {
        self.caches
            .application_ids_by_secret_id
            .insert(secret_id, application_ids)
            .await;
    }

    async fn invalidate_secret_bindings(&self, secret_id: Uuid) {
        self.caches
            .application_ids_by_secret_id
            .invalidate(&secret_id)
            .await;
        self.caches.application_ids_by_secret.invalidate_all();
    }

    async fn invalidate_secret(&self, secret_id: Uuid) {
        self.caches.by_id.invalidate(&secret_id).await;
        self.invalidate_secret_bindings(secret_id).await;
    }

    async fn invalidate_application_secrets(&self, application_id: Uuid) {
        self.caches.by_application.invalidate(&application_id).await;
    }
}

fn gen_secret() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

    let random = gen_random_with_charset(32, CHARSET);

    format!("app_{random}")
}
