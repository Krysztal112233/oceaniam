use oceaniam_database::model;
use oceaniam_vo::applications::ApplicationChallengeVO;

pub fn challenge_model_to_vo(model: model::challenges::Model) -> ApplicationChallengeVO {
    ApplicationChallengeVO {
        id: model.id,
        application_id: model.application_id.into(),
        subject_id: model.subject_id,
        factor_type: model.factor_type.to_string(),
        purpose: model.purpose.to_string(),
        status: model.status.to_string(),
        attempt_count: model.attempt_count,
        remaining_attempts: (model.max_attempts - model.attempt_count).max(0),
        expires_at: model.expires_at,
        consumed_at: model.consumed_at,
        created_at: model.created_at,
    }
}
