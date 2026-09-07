use chrono::{DateTime, FixedOffset};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::patch::PatchValue;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ToSchema)]
pub struct CreateOidcClientRequest {
    #[garde(length(min = 1, max = 128), custom(require_non_blank_oidc_client_name))]
    pub name: String,

    #[garde(length(min = 1, max = 100))]
    pub redirect_uris: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PatchOidcClientRequest {
    #[serde(default, skip_serializing_if = "PatchValue::is_missing")]
    #[schema(value_type = String)]
    pub name: PatchValue<String>,

    #[serde(default, skip_serializing_if = "PatchValue::is_missing")]
    #[schema(value_type = Vec<String>)]
    pub redirect_uris: PatchValue<Vec<String>>,
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

    // NOTE: AI-generated test
    #[test]
    fn oidc_client_patch_distinguishes_missing_null_and_values() {
        let missing: PatchOidcClientRequest =
            serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(missing, PatchOidcClientRequest::default());
        assert_eq!(
            serde_json::to_value(missing).unwrap(),
            serde_json::json!({})
        );

        let patch: PatchOidcClientRequest = serde_json::from_value(serde_json::json!({
            "name": null,
            "redirect_uris": ["https://client.example/callback"],
            "client_id": "immutable-value-is-ignored"
        }))
        .unwrap();
        assert!(matches!(patch.name, PatchValue::Null));
        assert!(matches!(
            patch.redirect_uris,
            PatchValue::Value(ref values)
                if values == &["https://client.example/callback".to_owned()]
        ));
    }
}
