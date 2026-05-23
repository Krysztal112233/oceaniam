use oceaniam_common::sqid::Sqid;
use oceaniam_database::model;
use oceaniam_vo::applications::{ApplicationDetailVO, ApplicationVO, CreateApplicationResponse};

use super::configurations::application_configuration_to_vo;

pub fn application_model_to_vo(model: model::applications::Model) -> ApplicationVO {
    let model::applications::Model {
        id,
        comment,
        tenant_id,
        ..
    } = model;
    ApplicationVO {
        id: id.into(),
        comment,
        tenant_id: tenant_id.into(),
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
        id: id.into(),
        comment,
        tenant_id: tenant_id.into(),
        configuration: serde_json::from_value::<
            oceaniam_database::config::application::ApplicationConfiguration,
        >(configuration)
        .map(application_configuration_to_vo)
        .unwrap(),
    }
}

pub fn create_application_response(
    id: Sqid,
    tenant_id: Sqid,
    comment: Option<String>,
) -> CreateApplicationResponse {
    CreateApplicationResponse {
        tenant_id,
        application_id: id,
        comment,
    }
}
