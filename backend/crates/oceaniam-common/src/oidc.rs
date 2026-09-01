use url::Url;

use crate::{config::PublicBaseUrl, sqid::Sqid};

/// Builds the canonical OIDC issuer URL for a tenant from the configured public origin.
pub fn tenant_oidc_issuer_url(public_base_url: &PublicBaseUrl, tenant_id: &Sqid) -> Url {
    let mut issuer = public_base_url.as_url().clone();
    issuer
        .path_segments_mut()
        .expect("validated HTTP(S) public base URL must support path segments")
        .clear()
        .push("oidc")
        .push(tenant_id.as_str());
    issuer
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn tenant_oidc_issuer_uses_the_dedicated_protocol_namespace() {
        let public_base_url =
            PublicBaseUrl::try_from(Url::parse("https://iam.example.com").unwrap()).unwrap();
        let tenant_id = Sqid::from(Uuid::from_u128(42));

        let issuer = tenant_oidc_issuer_url(&public_base_url, &tenant_id);

        assert_eq!(
            issuer.as_str(),
            format!("https://iam.example.com/oidc/{tenant_id}")
        );
        assert_eq!(
            public_base_url.as_url().as_str(),
            "https://iam.example.com/"
        );
    }
}
