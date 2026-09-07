use oceaniam_database::model::{
    oidc_client_redirect_uris, oidc_clients,
    sea_orm_active_enums::{OidcApplicationType, OidcClientType},
};
use oceaniam_vo::oidc_clients::{OidcApplicationTypeVO, OidcClientTypeVO, OidcClientVO};

use super::sqid::uuid_to_sqid;

pub fn oidc_client_model_to_vo(
    model: oidc_clients::Model,
    redirect_uris: Vec<oidc_client_redirect_uris::Model>,
) -> OidcClientVO {
    let mut redirect_uris = redirect_uris
        .into_iter()
        .map(|model| model.redirect_uri)
        .collect::<Vec<_>>();
    redirect_uris.sort_unstable();

    OidcClientVO {
        client_id: model.client_id,
        application_id: uuid_to_sqid(model.application_id),
        name: model.name,
        client_type: match model.client_type {
            OidcClientType::Public => OidcClientTypeVO::Public,
        },
        application_type: match model.application_type {
            OidcApplicationType::Web => OidcApplicationTypeVO::Web,
        },
        redirect_uris,
        created_at: model.created_at,
    }
}
