use oceaniam_database::model;
use oceaniam_vo::administrators::AdministratorVO;

pub fn administrator_model_to_vo(model: model::administrators::Model) -> AdministratorVO {
    let model::administrators::Model { id, name, .. } = model;
    AdministratorVO {
        id: id.into(),
        name,
    }
}
