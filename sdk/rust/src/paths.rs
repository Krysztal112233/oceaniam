use const_format::concatcp;

// Auth
pub const AUTH_TOKENS: &str = "/auth/tokens";
pub const AUTH_TOKENS_REFRESH: &str = concatcp!(AUTH_TOKENS, "/refresh");

// Tenants
pub const TENANTS: &str = "/tenants";
pub const TENANT: &str = concatcp!(TENANTS, "/{}");
pub const TENANT_USERS: &str = concatcp!(TENANT, "/users");

// Administrators
pub const ADMINISTRATORS: &str = "/administrators";
pub const ADMINISTRATOR: &str = concatcp!(ADMINISTRATORS, "/{}");

// Applications (tenant-scoped)
pub const TENANT_APPS: &str = concatcp!(TENANT, "/applications");
pub const TENANT_APP: &str = concatcp!(TENANT_APPS, "/{}");
pub const APP_CONFIG: &str = concatcp!(TENANT_APP, "/configuration");
pub const APP_USERS: &str = concatcp!(TENANT_APP, "/users");
pub const APP_USERS_SEARCH: &str = concatcp!(APP_USERS, "/search");
pub const APP_USER: &str = concatcp!(APP_USERS, "/{}");
pub const APP_USER_CREDS: &str = concatcp!(APP_USER, "/credentials");
pub const APP_TOKENS: &str = concatcp!(TENANT_APP, "/tokens");
pub const APP_TOKENS_REFRESH: &str = concatcp!(APP_TOKENS, "/refresh");
pub const APP_CHALLENGE: &str = concatcp!(TENANT_APP, "/challenges/{}");
pub const APP_KEYS: &str = concatcp!(TENANT_APP, "/keys");
pub const APP_KEY: &str = concatcp!(APP_KEYS, "/{}");

// JWKS (non-tenant)
pub const JWKS: &str = "/applications/{}/.well-known/jwks.json";

// Secrets
pub const SECRETS: &str = "/secrets";
pub const SECRET: &str = concatcp!(SECRETS, "/{}");

/// Replace `{}` placeholders with runtime arguments.
/// Number of `{}` in template must match the number of args.
pub fn fmt1(template: &str, a: &str) -> String {
    template.replacen("{}", a, 1)
}

pub fn fmt2(template: &str, a: &str, b: &str) -> String {
    template.replacen("{}", a, 1).replacen("{}", b, 1)
}

pub fn fmt3(template: &str, a: &str, b: &str, c: &str) -> String {
    template
        .replacen("{}", a, 1)
        .replacen("{}", b, 1)
        .replacen("{}", c, 1)
}
