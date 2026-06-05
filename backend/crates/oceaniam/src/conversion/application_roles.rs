use oceaniam_database::model;
use oceaniam_vo::application_roles::ApplicationRoleVO;

pub fn application_role_model_to_vo(
    model: model::application_roles::Model,
    permissions: Vec<String>,
) -> ApplicationRoleVO {
    ApplicationRoleVO {
        id: model.id,
        application_id: model.application_id,
        name: model.name,
        is_system: model.is_system,
        permissions,
    }
}
