use oceaniam_database::model;
use oceaniam_vo::applications::ApplicationUserVO;

pub fn user_model_to_vo(model: model::users::Model) -> ApplicationUserVO {
    let model::users::Model {
        id,
        email,
        phone,
        nickname,
        ..
    } = model;
    ApplicationUserVO {
        id: id.into(),
        email,
        phone,
        nickname,
    }
}
