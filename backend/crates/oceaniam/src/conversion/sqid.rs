use std::str::FromStr;

use oceaniam_common::sqid::Sqid;
use uuid::Uuid;

pub fn uuid_to_sqid(uuid: Uuid) -> String {
    Sqid::from(uuid).into_inner()
}

pub fn sqid_to_uuid(input: &str) -> Result<Uuid, oceaniam_common::sqid::Error> {
    Sqid::from_str(input).and_then(Uuid::try_from)
}

#[cfg(test)]
mod tests {
    use super::{sqid_to_uuid, uuid_to_sqid};
    use uuid::Uuid;

    // NOTE: AI-generated test
    #[test]
    fn uuid_to_sqid_round_trips_to_uuid() {
        let uuid = Uuid::now_v7();
        let sqid = uuid_to_sqid(uuid);

        let decoded = sqid_to_uuid(&sqid).unwrap();

        assert_eq!(decoded, uuid);
    }

    // NOTE: AI-generated test
    #[test]
    fn sqid_to_uuid_rejects_invalid_input() {
        let result = sqid_to_uuid("not a sqid");

        assert!(result.is_err());
    }
}
