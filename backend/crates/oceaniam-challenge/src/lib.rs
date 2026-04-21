use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub trait ChallengePayload: Serialize + DeserializeOwned {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpPayload {}
