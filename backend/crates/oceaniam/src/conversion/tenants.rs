use oceaniam_database::model;
use oceaniam_vo::tenants::TenantVO;

use super::sqid::uuid_to_sqid;

pub fn tenant_model_to_vo(model: model::tenants::Model) -> TenantVO {
    let model::tenants::Model { id, comment, .. } = model;
    TenantVO {
        id: uuid_to_sqid(id),
        comment,
    }
}
