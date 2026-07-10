use std::sync::{Arc, OnceLock};
use std::time::Duration;

use crate::error::Error;
use argon2::{Argon2, Params};
use axum::http::StatusCode;
use futures::future::join_all;
use moka::future::Cache;
use oceaniam_database::{
    config::application::ApplicationConfiguration,
    helper::{
        SafeTransactionConnectionTrait,
        applications::{ApplicationHelper, CreateApplicationOptions},
        users::{CreateUserOpts, CreateUserResult, PatchUserOpts, UserContactOpts, UserHelper},
    },
    model::{
        applications::Model as ApplicationModel,
        prelude::{Applications, Users},
        users::Model as UserModel,
    },
};
use oceaniam_vo::applications::{PatchApplicationConfigurationRequest, PatchApplicationRequest};
use oceaniam_vo::auth::AuthVO;
use oceaniam_vo::patch::PatchValue;
use sea_orm::prelude::*;
use tap::Tap;
use tracing::{error, info};
use uuid::Uuid;

use crate::state::audit::Auditing;
use crate::state::challenge::ManagedChallenges;
use crate::state::credentials::ManagedCredentialVaults;
use crate::state::secret::Secrets;

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

/// Per-application scope that bundles configuration, user state, and challenge manager together.
///
/// Constructed lazily on first access to any sub-state.  Invalidating the single
/// [`Cache`] entry for an `application_id` atomically evicts all three.
///
/// `users` and `challenges` are initialized on first access via [`OnceLock`], so accessing only
/// [`ApplicationScope::configuration`] does not pay the cost of constructing the other two.
struct ApplicationScope {
    application_id: Uuid,
    configuration: ApplicationConfiguration,

    database: DatabaseConnection,
    shared_credential_vaults: ManagedCredentialVaults,
    auditing: Auditing,

    users: OnceLock<Arc<ApplicationUsers>>,
    challenges: OnceLock<Arc<ManagedChallenges>>,
}

impl std::fmt::Debug for ApplicationScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationScope")
            .field("application_id", &self.application_id)
            .field("configuration", &self.configuration)
            .field("users", &self.users.get())
            .field("challenges", &self.challenges.get())
            .finish()
    }
}

impl ApplicationScope {
    fn users(&self) -> Arc<ApplicationUsers> {
        self.users
            .get_or_init(|| {
                Arc::new(ApplicationUsers::new(
                    self.application_id,
                    self.shared_credential_vaults.clone(),
                    self.database.clone(),
                    self.configuration.clone(),
                ))
            })
            .clone()
    }

    fn challenges(&self) -> Arc<ManagedChallenges> {
        self.challenges
            .get_or_init(|| {
                Arc::new(ManagedChallenges::new(
                    self.application_id,
                    self.database.clone(),
                    self.auditing.clone(),
                    self.shared_credential_vaults.clone(),
                ))
            })
            .clone()
    }
}

/// Central manager for application-level operations.
///
/// Provides CRUD for applications, per-application user management, secret binding, configuration
/// patching, and MFA challenge handling.  Internal per-application state is backed by a single
/// [`Cache<Uuid, Arc<ApplicationScope>>`] that bundles configuration, users, and challenges
/// together — constructed lazily on first access and atomically invalidated on write.
///
/// - **Users**: cached with a 30-min idle TTL; password verification delegates to [`ManagedCredentialVaults`].
/// - **Configurations**: cached with a 30-min idle TTL; re-fetched on write.
/// - **Challenges**: created per-application via [`ManagedChallenges`]; cached with a 30-min idle TTL.
/// - **Secrets**: see [`Secrets`] for `app_xxx` binding.
#[derive(Debug, Clone)]
pub struct ManagedApplications {
    database: DatabaseConnection,

    secrets: Secrets,

    applications: Cache<Uuid, Arc<ApplicationScope>>,

    /// Cache for application models (basic entity data).
    ///
    /// Invalidation: see `delete_application`, `patch_application`, `patch_configuration`.
    models: Cache<Uuid, Arc<ApplicationModel>>,

    /// This field are shared with global states.
    shared_credential_vaults: ManagedCredentialVaults,

    auditing: Auditing,
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

impl ManagedApplications {
    pub fn new(
        credential: ManagedCredentialVaults,
        database: DatabaseConnection,
        auditing: Auditing,
    ) -> ManagedApplications {
        ManagedApplications {
            database: database.clone(),
            secrets: Secrets::new(database.clone()),
            applications: Cache::builder()
                .time_to_idle(Duration::from_mins(30))
                .build(),

            models: Cache::builder()
                .time_to_idle(Duration::from_mins(30))
                .build(),

            shared_credential_vaults: credential,
            auditing,
        }
    }

    pub async fn get_application_users(
        &self,
        application_id: Uuid,
    ) -> Result<Arc<ApplicationUsers>, Error> {
        Ok(self.get_or_init_scope(application_id).await?.users())
    }

    pub async fn get_model(&self, application_id: Uuid) -> Result<Arc<ApplicationModel>, Error> {
        self.is_application_exist(application_id).await?;

        let database = self.database.clone();

        self.models
            .try_get_with(application_id, async move {
                Applications::get_application(application_id, &database)
                    .await
                    .map(Arc::new)
            })
            .await
            .map_err(|e| Error::Internal {
                msg: e.to_string(),
                location: snafu::location!(),
            })
    }

    #[allow(private_bounds)]
    pub async fn find_user_by(
        &self,
        application_id: Uuid,
        user_identifier: impl Into<UserIdentifier> + Send,
    ) -> Result<UserModel, Error> {
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

        self.applications.invalidate(&application_id).await;
        self.models.invalidate(&application_id).await;

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

        Ok(model)
    }

    pub async fn get_configuration(
        &self,
        application_id: Uuid,
    ) -> Result<ApplicationConfiguration, Error> {
        Ok(self
            .get_or_init_scope(application_id)
            .await?
            .configuration
            .clone())
    }

    pub async fn patch_configuration(
        &self,
        application_id: Uuid,
        patch: PatchApplicationConfigurationRequest,
    ) -> Result<ApplicationConfiguration, Error> {
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

        self.applications.invalidate(&application_id).await;
        self.models.invalidate(&application_id).await;

        Ok(configuration)
    }

    pub async fn patch_application(
        &self,
        application_id: Uuid,
        patch: PatchApplicationRequest,
    ) -> Result<ApplicationModel, Error> {
        let is_missing = matches!(patch.comment, PatchValue::Missing);

        let result: ApplicationModel = match patch.comment {
            PatchValue::Missing => {
                Applications::get_application(application_id, &self.database).await?
            }
            PatchValue::Null => {
                Applications::update_comment(application_id, None, &self.database).await?
            }
            PatchValue::Value(comment) => {
                Applications::update_comment(application_id, Some(comment), &self.database).await?
            }
        };

        if !is_missing {
            self.models.invalidate(&application_id).await;
        }

        Ok(result)
    }

    async fn get_or_init_scope(
        &self,
        application_id: Uuid,
    ) -> Result<Arc<ApplicationScope>, Error> {
        let database = self.database.clone();
        let shared_credential_vaults = self.shared_credential_vaults.clone();
        let auditing = self.auditing.clone();

        Ok(self
            .applications
            .try_get_with(application_id, async move {
                let configuration: ApplicationConfiguration =
                    Applications::get_application(application_id, &database)
                        .await?
                        .into();

                Ok(Arc::new(ApplicationScope {
                    application_id,
                    configuration,
                    database,
                    shared_credential_vaults,
                    auditing,
                    users: OnceLock::new(),
                    challenges: OnceLock::new(),
                }))
            })
            .await?)
    }

    async fn is_application_exist(&self, application_id: Uuid) -> Result<(), Error> {
        if Applications::is_exist(application_id, &self.database).await? {
            Ok(())
        } else {
            Err(Error::with_code(
                StatusCode::NOT_FOUND,
                format!("application_id={application_id} doesn't exist"),
            ))
        }
    }
}

impl ManagedApplications {
    pub fn secrets(&self) -> &Secrets {
        &self.secrets
    }

    pub async fn challenges(&self, application_id: Uuid) -> Result<Arc<ManagedChallenges>, Error> {
        Ok(self.get_or_init_scope(application_id).await?.challenges())
    }
}

/// Per-application user cache and credential operations.
///
/// Existence is guaranteed by [`ManagedApplications::get_or_init_scope`], which constructs
/// [`ApplicationScope`] only after confirming the application exists.
#[derive(Debug, Clone)]
pub struct ApplicationUsers {
    application_id: Uuid,
    configuration: ApplicationConfiguration,

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
    fn new(
        application_id: Uuid,
        shared_credential_vaults: ManagedCredentialVaults,
        database: DatabaseConnection,
        configuration: ApplicationConfiguration,
    ) -> Self {
        Self {
            application_id,
            database,
            cache: Cache::builder()
                .time_to_idle(Duration::from_secs(30))
                .build(),
            shared_credential_vaults,
            configuration,
        }
    }

    pub async fn find_user_by(&self, user_identifier: UserIdentifier) -> Result<UserModel, Error> {
        Ok(self
            .cache
            .try_get_with(user_identifier.clone(), async move {
                let result = match user_identifier {
                    UserIdentifier::Email(mail) => {
                        Users::find_contact_user(
                            self.application_id,
                            UserContactOpts {
                                email: Some(mail),
                                phone: None,
                            },
                            &self.database,
                        )
                        .await
                    }
                    UserIdentifier::Phone(phone) => {
                        Users::find_contact_user(
                            self.application_id,
                            UserContactOpts {
                                email: None,
                                phone: Some(phone),
                            },
                            &self.database,
                        )
                        .await
                    }
                    UserIdentifier::Id(uuid) => {
                        Users::get_user_of_application(self.application_id, uuid, &self.database)
                            .await
                    }
                };
                result.map_err(Into::into)
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
        let argon2 = build_argon2(&self.configuration)?;

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

    pub async fn patch_user(
        &self,
        application_id: Uuid,
        user_id: Uuid,
        patched: PatchUserOpts,
    ) -> Result<UserModel, Error> {
        let user = Users::patch_user(application_id, user_id, patched, &self.database)
            .await
            .inspect_err(|e| {
                error!(
                    "failed to patch user: user_id={}, application_id={}, error={}",
                    user_id, application_id, e
                );
            })?;

        self.cache
            .insert(UserIdentifier::Id(user_id), user.clone())
            .await;

        join_all(
            [
                user.email.clone().map(UserIdentifier::Email),
                user.phone.clone().map(UserIdentifier::Phone),
            ]
            .iter()
            .flatten()
            .map(|it| self.cache.invalidate(it)),
        )
        .await;

        Ok(user)
    }

    pub async fn delete_user(&self, application_id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.delete_user_in_tx(application_id, user_id, &self.database)
            .await
    }

    /// Deletes a user within a shared transaction.
    ///
    /// Deletion order is critical: `users.id -> subjects.id` is `ON DELETE NO ACTION`, so the
    /// `users` row must be removed first. The credential is then dropped via
    /// [`ManagedCredentialVaults::drop_credential_in_tx`], which cascades to `subjects` and
    /// `subject_roles` at the database level and evicts the credential cache. Finally the
    /// per-application user cache entries (`Id`, `Email`, `Phone`) are invalidated.
    pub async fn delete_user_in_tx(
        &self,
        application_id: Uuid,
        user_id: Uuid,
        transaction: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        info!(
            "deleting application user: user_id={}, application_id={}",
            user_id, application_id
        );

        let tx = transaction.begin().await?;

        // Fetch the user model first so we can invalidate email/phone cache entries.
        // Reuse the moka cache (handler should have already populated it); avoids an extra DB hit.
        let user = self.find_user_by(UserIdentifier::Id(user_id)).await?;

        Users::delete_user(user_id, &tx).await.inspect_err(|e| {
            error!(
                "failed to delete user: user_id={}, application_id={}, error={}",
                user_id, application_id, e
            );
        })?;

        self.shared_credential_vaults
            .drop_credential_in_tx(user_id, &tx)
            .await
            .inspect_err(|e| {
                error!(
                    "failed to delete credential for user: user_id={}, application_id={}, error={}",
                    user_id, application_id, e
                );
            })?;
        tx.commit().await?;

        self.cache.remove(&UserIdentifier::Id(user_id)).await;
        if let Some(email) = user.email {
            self.cache.remove(&UserIdentifier::Email(email)).await;
        }
        if let Some(phone) = user.phone {
            self.cache.remove(&UserIdentifier::Phone(phone)).await;
        }

        info!(
            "application user deleted successfully: user_id={}, application_id={}",
            user_id, application_id
        );

        Ok(())
    }
}
