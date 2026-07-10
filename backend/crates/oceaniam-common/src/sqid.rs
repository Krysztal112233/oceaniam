use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use snafu::{Location, Snafu};
use sqids::Sqids;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid sqid at {location}"))]
    InvalidSqid { location: Location },
}

static SQID: LazyLock<sqids::Sqids> = LazyLock::new(|| Sqids::new(None).unwrap());

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[repr(transparent)]
pub struct Sqid(String);

impl Sqid {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<Uuid> for Sqid {
    fn from(value: Uuid) -> Self {
        let bytes = value.as_u128();
        let high = (bytes >> 64) as u64;
        let low = bytes as u64;

        let encoded = SQID
            .encode(&[high, low])
            .expect("sqids encoding should not fail");

        Self(encoded)
    }
}

impl TryFrom<Sqid> for Uuid {
    type Error = Error;

    fn try_from(value: Sqid) -> Result<Self, Self::Error> {
        let decoded = SQID.decode(&value.0);

        if decoded.len() != 2 {
            return Err(Error::InvalidSqid {
                location: snafu::location!(),
            });
        }

        let high = (decoded[0] as u128) << 64;
        let low = decoded[1] as u128;

        Ok(Uuid::from_u128(high | low))
    }
}

impl FromStr for Sqid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = SQID.decode(s);

        if decoded.len() != 2 {
            return Err(Error::InvalidSqid {
                location: snafu::location!(),
            });
        }

        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for Sqid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Sqid {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn test_encode_decode_success() {
        let uuid = Uuid::now_v7();
        let sqid: Sqid = uuid.into();
        let decoded: Uuid = sqid.try_into().unwrap();
        assert_eq!(uuid, decoded);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_decode_invalid_sqid() {
        let invalid_sqid_str = "!@#$%^&*";
        let result: Result<Sqid, _> = invalid_sqid_str.parse();
        assert!(result.is_err());
    }
}
