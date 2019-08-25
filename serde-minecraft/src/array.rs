//! Serialize fixed-size arrays to a minecraft-compatible format.
//!
//! Serde, by default, serializes arrays with a fixed size (e. g. `[u8; 16]`) as
//! tuples. This library, however, serializes tuples without a size specification
//! on the wire to allow for things like `struct Vec3(f32, f32, f32);` to work.
//!
//! As such, when using fixed-size arrays to store minecraft-network-format arrays,
//! the format specification is violated. This module fixes that.
//!
//! Use this module via `#[serde(with = "serde_minecraft::array")]`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::iter::FromIterator;

pub fn deserialize<'de, I, T, D>(deserializer: D) -> Result<T, D::Error>
where
    I: Deserialize<'de>,
    T: FromIterator<I>,
    D: Deserializer<'de>,
{
    let values = Vec::deserialize(deserializer)?;
    Ok(T::from_iter(values))
}

pub fn serialize<T, S>(arr: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: IntoIterator,
    T::Item: Serialize,
    S: Serializer,
{
    serializer.collect_seq(arr)
}
