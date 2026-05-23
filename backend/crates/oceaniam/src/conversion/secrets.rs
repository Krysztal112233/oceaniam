use oceaniam_database::model;
use oceaniam_vo::applications::SecretVO;

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
        id: model.id.into(),
        secret: model.secret.clone(),
        created_at: model.created_at.to_rfc2822(),
        revoked_at: model.revoked_at.map(|it| it.to_rfc2822()),
        application_ids: Vec::new(),
    }
}
