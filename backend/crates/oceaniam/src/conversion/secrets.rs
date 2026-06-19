use oceaniam_database::model;
use oceaniam_vo::applications::SecretVO;
use uuid::Uuid;

use super::sqid::uuid_to_sqid;

pub fn secret_with_masked(model: model::application_secrets::Model) -> SecretVO {
    let mut vo = secret_with_unmasked_inner(&model);
    vo.secret = format!("{}...", model.secret.split_at(8).0);
    vo
}

pub fn secret_with_unmasked(model: model::application_secrets::Model) -> SecretVO {
    secret_with_unmasked_inner(&model)
}

fn secret_with_unmasked_inner(model: &model::application_secrets::Model) -> SecretVO {
    SecretVO {
        id: uuid_to_sqid(model.id),
        secret: model.secret.clone(),
        created_at: model.created_at.to_rfc2822(),
        revoked_at: model.revoked_at.map(|it| it.to_rfc2822()),
        application_ids: Vec::new(),
    }
}

pub fn with_application_ids(
    mut secret: SecretVO,
    application_ids: impl IntoIterator<Item = Uuid>,
) -> SecretVO {
    secret.application_ids = application_ids.into_iter().map(uuid_to_sqid).collect();
    secret
}
