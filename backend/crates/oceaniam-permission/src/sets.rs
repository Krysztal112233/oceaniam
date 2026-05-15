use std::collections::HashSet;
use std::sync::LazyLock;

use tap::Tap;

use crate::permission::Permission;

// ── Application role permission sets (least → most) ──

pub(crate) static APP_READER_PERMS: LazyLock<HashSet<Permission>> = LazyLock::new(|| {
    use Permission::*;
    HashSet::from([ApplicationUserRead, ApplicationChallengeRead])
});

pub(crate) static APP_MEMBER_PERMS: LazyLock<HashSet<Permission>> = LazyLock::new(|| {
    use Permission::*;
    APP_READER_PERMS.clone().tap_mut(|it| {
        it.insert(ApplicationTokenIssue);
    })
});

pub(crate) static APP_ADMIN_PERMS: LazyLock<HashSet<Permission>> = LazyLock::new(|| {
    use Permission::*;
    APP_MEMBER_PERMS.clone().tap_mut(|it| {
        it.extend([
            ApplicationUserInvite,
            ApplicationUserPatch,
            ApplicationTokenRevoke,
        ])
    })
});

pub(crate) static APP_OWNER_PERMS: LazyLock<HashSet<Permission>> = LazyLock::new(|| {
    use Permission::*;
    APP_ADMIN_PERMS.clone().tap_mut(|it| {
        it.insert(ApplicationUserDelete);
    })
});

// ── Platform role permission sets ──

#[rustfmt::skip]
pub(crate) static PLATFORM_READ_BASE: LazyLock<HashSet<Permission>> = LazyLock::new(|| {
    use Permission::*;
    HashSet::from([
        TenantRead,

        ApplicationRead,

        ApplicationConfigurationRead,

        SecretRead,

        KeyRead,
    ])
});

pub(crate) static PLATFORM_READONLY_ADMIN_PERMS: LazyLock<HashSet<Permission>> =
    LazyLock::new(|| {
        use Permission::*;
        PLATFORM_READ_BASE.clone().tap_mut(|it| {
            it.insert(AdministratorRead);
        })
    });

#[rustfmt::skip]
pub(crate) static PLATFORM_TENANT_ADMIN_PERMS: LazyLock<HashSet<Permission>> = LazyLock::new(|| {
    use Permission::*;
    PLATFORM_READ_BASE.clone().tap_mut(|it| {
        it.extend([
            TenantPatch,

            ApplicationCreate,
            ApplicationPatch,
            ApplicationDelete,
            ApplicationConfigurationPatch,

            SecretCreate,
            SecretDelete,

            KeyRotate,
            KeyRevoke,
        ])
    })
});

#[rustfmt::skip]
pub(crate) static PLATFORM_SUPER_ADMIN_PERMS: LazyLock<HashSet<Permission>> = LazyLock::new(|| {
    use Permission::*;
    PLATFORM_TENANT_ADMIN_PERMS.clone().tap_mut(|it| {
        it.extend([
            TenantCreate,
            TenantDelete,

            AdministratorRead,
            AdministratorCreate,
            AdministratorPatch,
        ])
    })
});
