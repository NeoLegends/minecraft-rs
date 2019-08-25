use serde::{Deserialize, Deserializer, Serializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let unsigned = u64::deserialize(deserializer)?;
    Ok(unsigned as i64)
}

pub fn serialize<S>(val: i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(val as u64)
}
