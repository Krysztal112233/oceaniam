use oceaniam_database::model;
use oceaniam_vo::applications::ApplicationKeyVO;

use super::sqid::uuid_to_sqid;

pub fn key_model_to_vo(model: model::key_boxes::Model) -> ApplicationKeyVO {
    let model::key_boxes::Model {
        id,
        key_alg,
        status,
        created_at,
        activated_at,
        retired_at,
        expires_at,
        revoked_at,
        ..
    } = model;
    ApplicationKeyVO {
        key_id: uuid_to_sqid(id),
        algorithm: key_alg.to_string(),
        status: status.to_string(),
        created_at,
        activated_at,
        retired_at,
        expires_at,
        revoked_at,
    }
}
