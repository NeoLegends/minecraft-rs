//! Serde support for the minecraft network format.
//!
//! This crate supports structs, tuples, tuple structs, units, and unit enums.
//! Enums with data (this includes `Option<T>` and `Result<T, E>`) are _not_
//! supported, because the network format does not model that.
//!
//! Minecraft's network format is not self-describing. Therefore, when using
//! serde's derive feature, ensure your struct members are listed in the same
//! order as specified in the format specification. Otherwise the library
//! will yield garbage values.

use bytes::{Buf, BufMut, BytesMut};
use serde::{de::Deserialize, ser::Serialize};

pub mod array;
pub mod fixed_i32;
pub mod fixed_i64;

pub use error::*;

mod de;
mod error;
mod ser;
mod size;

/// Attempts to deserialize a `T` in minecraft network format from the given `Buf`.
pub fn read_from<'de, T: Deserialize<'de>, B: Buf>(
    buf: &'de mut B,
) -> Result<T, Error> {
    let mut deserializer = de::Deserializer::new(buf);
    T::deserialize(&mut deserializer)
}

/// Computes the size of the given `T` if it was written to minecraft network
/// format.
///
/// Minecraft uses length-prefixed network messages. As such it may come in handy
/// to know the size of a message before serializing it to a buffer, in order to
/// avoid having to shift around the data after serialization.
pub fn serialized_size<T: Serialize>(val: &T) -> Result<usize, Error> {
    let mut computor = size::ComputeSize::new();
    val.serialize(&mut computor)?;
    Ok(computor.size())
}

/// Writes the given `T` into the given `BytesMut`, reserving space as necessary.
pub fn write_to<T: Serialize>(val: &T, buf: &mut BytesMut) -> Result<(), Error> {
    let size = serialized_size(val)?;
    buf.reserve(size);
    write_to_no_resize(val, buf)
}

/// Writes the given `T` into the given `BufMut`, without reserving space
/// beforehand.
///
/// This function spanic if the given `BufMut` does not have the required capacity.
pub fn write_to_no_resize<T: Serialize, B: BufMut>(
    val: &T,
    buf: &mut B,
) -> Result<(), Error> {
    let mut serializer = ser::Serializer::new(buf);
    val.serialize(&mut serializer)?;
    Ok(())
}
