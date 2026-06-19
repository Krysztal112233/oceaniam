use oceaniam_database::model;
use oceaniam_vo::administrators::AdministratorVO;

use super::sqid::uuid_to_sqid;

pub fn administrator_model_to_vo(model: model::administrators::Model) -> AdministratorVO {
    let model::administrators::Model { id, name, .. } = model;
    AdministratorVO {
        id: uuid_to_sqid(id),
        name,
    }
}
