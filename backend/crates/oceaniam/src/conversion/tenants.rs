use oceaniam_database::model;
use oceaniam_vo::tenants::TenantVO;

pub fn tenant_model_to_vo(model: model::tenants::Model) -> TenantVO {
    let model::tenants::Model { id, comment, .. } = model;
    TenantVO {
        id: id.into(),
        comment,
    }
}
