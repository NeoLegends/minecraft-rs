//! By default, this library serializes `i32`s using minecraft's VarInt encoding.
//!
//! Use this module via `#[serde(with = "serde_minecraft::fixed_i32")]` to
//! circumvent this. The `i32` in question will be serialized to 4 bytes in big
//! endian order.

use serde::{Deserialize, Deserializer, Serializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let unsigned = u32::deserialize(deserializer)?;
    Ok(unsigned as i32)
}

pub fn serialize<S>(val: &i32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u32(*val as u32)
}
