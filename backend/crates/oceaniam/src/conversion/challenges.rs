use oceaniam_database::model::{
    self,
    sea_orm_active_enums::{ChallengeFactorType, ChallengePurposeType, ChallengeStatusType},
};
use oceaniam_vo::applications::ApplicationChallengeVO;

pub fn challenge_model_to_vo(model: model::challenges::Model) -> ApplicationChallengeVO {
    ApplicationChallengeVO {
        id: model.id,
        application_id: model.application_id.into(),
        subject_id: model.subject_id,
        factor_type: challenge_factor_type_to_str(&model.factor_type).to_owned(),
        purpose: challenge_purpose_type_to_str(&model.purpose).to_owned(),
        status: challenge_status_type_to_str(&model.status).to_owned(),
        attempt_count: model.attempt_count,
        remaining_attempts: (model.max_attempts - model.attempt_count).max(0),
        expires_at: model.expires_at,
        consumed_at: model.consumed_at,
        created_at: model.created_at,
    }
}

fn challenge_factor_type_to_str(factor_type: &ChallengeFactorType) -> &'static str {
    match factor_type {
        ChallengeFactorType::EmailTotp => "email_totp",
        ChallengeFactorType::Totp => "totp",
    }
}

fn challenge_purpose_type_to_str(purpose: &ChallengePurposeType) -> &'static str {
    match purpose {
        ChallengePurposeType::Signin => "signin",
    }
}

fn challenge_status_type_to_str(status: &ChallengeStatusType) -> &'static str {
    match status {
        ChallengeStatusType::Pending => "pending",
        ChallengeStatusType::Consumed => "consumed",
    }
}
