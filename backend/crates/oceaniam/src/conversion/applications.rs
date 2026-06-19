use oceaniam_database::model;
use oceaniam_vo::applications::{ApplicationDetailVO, ApplicationVO, CreateApplicationResponse};
use uuid::Uuid;

use super::configurations::application_configuration_to_vo;
use super::sqid::uuid_to_sqid;

pub fn application_model_to_vo(model: model::applications::Model) -> ApplicationVO {
    let model::applications::Model {
        id,
        comment,
        tenant_id,
        ..
    } = model;
    ApplicationVO {
        id: uuid_to_sqid(id),
        comment,
        tenant_id: uuid_to_sqid(tenant_id),
    }
}

pub fn application_detail_model_to_vo(model: model::applications::Model) -> ApplicationDetailVO {
    let model::applications::Model {
        id,
        comment,
        tenant_id,
        configuration,
        ..
    } = model;
    ApplicationDetailVO {
        id: uuid_to_sqid(id),
        comment,
        tenant_id: uuid_to_sqid(tenant_id),
        configuration: serde_json::from_value::<
            oceaniam_database::config::application::ApplicationConfiguration,
        >(configuration)
        .map(application_configuration_to_vo)
        .unwrap(),
    }
}

pub fn create_application_response(
    id: Uuid,
    tenant_id: Uuid,
    comment: Option<String>,
) -> CreateApplicationResponse {
    CreateApplicationResponse {
        tenant_id: uuid_to_sqid(tenant_id),
        application_id: uuid_to_sqid(id),
        comment,
    }
}
