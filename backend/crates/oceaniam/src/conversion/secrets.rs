use oceaniam_application_secret::masked_from_stored_prefix;
use oceaniam_database::model;
use oceaniam_vo::applications::SecretVO;
use uuid::Uuid;

use super::sqid::uuid_to_sqid;

pub fn secret_with_masked(model: model::application_secrets::Model) -> SecretVO {
    let masked = masked_from_stored_prefix(&model.secret_prefix)
        .expect("stored application secret prefix must have the validated format");
    secret_with_value(&model, masked)
}

pub fn secret_with_plaintext(
    model: model::application_secrets::Model,
    plaintext: &str,
) -> SecretVO {
    secret_with_value(&model, plaintext.to_owned())
}

fn secret_with_value(model: &model::application_secrets::Model, secret: String) -> SecretVO {
    SecretVO {
        id: uuid_to_sqid(model.id),
        secret,
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
