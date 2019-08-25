//! By default, this library serializes `i64`s using minecraft's VarInt encoding.
//!
//! Use this module via `#[serde(with = "serde_minecraft::fixed_i64")]` to
//! circumvent this. The `i64` in question will be serialized to 8 bytes in big
//! endian order.

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
