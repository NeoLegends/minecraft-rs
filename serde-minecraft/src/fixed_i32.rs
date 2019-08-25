use serde::{Deserialize, Deserializer, Serializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let unsigned = u32::deserialize(deserializer)?;
    Ok(unsigned as i32)
}

pub fn serialize<S>(val: i32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u32(val as u32)
}
