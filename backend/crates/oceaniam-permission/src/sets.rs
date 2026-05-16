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

#[cfg(test)]
mod tests {
    use super::*;

    /// All 26 permission variants. Keep in sync with [`Permission`].
    const ALL_PERMISSIONS: [Permission; 26] = [
        Permission::TenantCreate,
        Permission::TenantDelete,
        Permission::TenantPatch,
        Permission::TenantRead,
        Permission::ApplicationCreate,
        Permission::ApplicationDelete,
        Permission::ApplicationPatch,
        Permission::ApplicationRead,
        Permission::ApplicationConfigurationPatch,
        Permission::ApplicationConfigurationRead,
        Permission::SecretCreate,
        Permission::SecretDelete,
        Permission::SecretRead,
        Permission::KeyRead,
        Permission::KeyRevoke,
        Permission::KeyRotate,
        Permission::AdministratorCreate,
        Permission::AdministratorPatch,
        Permission::AdministratorRead,
        Permission::ApplicationUserDelete,
        Permission::ApplicationUserInvite,
        Permission::ApplicationUserPatch,
        Permission::ApplicationUserRead,
        Permission::ApplicationTokenIssue,
        Permission::ApplicationTokenRevoke,
        Permission::ApplicationChallengeRead,
    ];

    // NOTE: AI-generated test
    #[test]
    fn every_permission_appears_in_at_least_one_role() {
        let all_variants: HashSet<Permission> = HashSet::from(ALL_PERMISSIONS);

        let covered: HashSet<Permission> = [
            &*PLATFORM_SUPER_ADMIN_PERMS,
            &*PLATFORM_TENANT_ADMIN_PERMS,
            &*PLATFORM_READONLY_ADMIN_PERMS,
            &*APP_OWNER_PERMS,
            &*APP_ADMIN_PERMS,
            &*APP_MEMBER_PERMS,
            &*APP_READER_PERMS,
        ]
        .into_iter()
        .flatten()
        .copied()
        .collect();

        assert_eq!(
            all_variants, covered,
            "every Permission variant must appear in at least one role set"
        );
    }
}
