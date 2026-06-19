use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum PatchValue<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}

impl<'de, T> Deserialize<'de> for PatchValue<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Option::<T>::deserialize(deserializer)? {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

impl<T: Serialize> Serialize for PatchValue<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Missing => serializer.serialize_none(),
            Self::Null => serializer.serialize_none(),
            Self::Value(v) => v.serialize(serializer),
        }
    }
}

impl<T> PatchValue<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::PatchValue;

    #[derive(Debug, Deserialize, Serialize)]
    struct PatchFixture {
        #[serde(default, skip_serializing_if = "PatchValue::is_missing")]
        value: PatchValue<String>,
    }

    // NOTE: AI-generated test
    #[test]
    fn patch_value_distinguishes_missing_null_and_value() {
        let missing: PatchFixture = serde_json::from_str("{}").unwrap();
        let null: PatchFixture = serde_json::from_str(r#"{"value":null}"#).unwrap();
        let value: PatchFixture = serde_json::from_str(r#"{"value":"updated"}"#).unwrap();

        assert!(matches!(missing.value, PatchValue::Missing));
        assert!(matches!(null.value, PatchValue::Null));
        assert!(matches!(value.value, PatchValue::Value(ref v) if v == "updated"));
    }

    // NOTE: AI-generated test
    #[test]
    fn missing_patch_value_is_skipped_when_serialized() {
        let fixture = PatchFixture {
            value: PatchValue::Missing,
        };

        let serialized = serde_json::to_string(&fixture).unwrap();

        assert_eq!(serialized, "{}");
    }
}
