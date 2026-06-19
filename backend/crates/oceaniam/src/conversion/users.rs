use oceaniam_database::model;
use oceaniam_vo::applications::ApplicationUserVO;

use super::sqid::uuid_to_sqid;

pub fn user_model_to_vo(model: model::users::Model) -> ApplicationUserVO {
    let model::users::Model {
        id,
        email,
        phone,
        nickname,
        ..
    } = model;
    ApplicationUserVO {
        id: uuid_to_sqid(id),
        email,
        phone,
        nickname,
    }
}
