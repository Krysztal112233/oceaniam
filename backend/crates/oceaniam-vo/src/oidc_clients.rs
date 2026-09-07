use chrono::{DateTime, FixedOffset};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ToSchema)]
pub struct CreateOidcClientRequest {
    #[garde(length(min = 1, max = 128), custom(require_non_blank_oidc_client_name))]
    pub name: String,

    #[garde(length(min = 1, max = 100))]
    pub redirect_uris: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum OidcClientTypeVO {
    Public,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum OidcApplicationTypeVO {
    Web,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct OidcClientVO {
    pub client_id: String,
    pub application_id: String,
    pub name: String,
    pub client_type: OidcClientTypeVO,
    pub application_type: OidcApplicationTypeVO,
    pub redirect_uris: Vec<String>,
    pub created_at: DateTime<FixedOffset>,
}

fn require_non_blank_oidc_client_name(value: &str, _: &()) -> garde::Result {
    if value.trim().is_empty() {
        return Err(garde::Error::new("must not be blank"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn oidc_client_wire_enums_are_lowercase() {
        assert_eq!(
            serde_json::to_value(OidcClientTypeVO::Public).unwrap(),
            serde_json::json!("public")
        );
        assert_eq!(
            serde_json::to_value(OidcApplicationTypeVO::Web).unwrap(),
            serde_json::json!("web")
        );
    }
}
