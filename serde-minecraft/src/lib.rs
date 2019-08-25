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

pub fn read_from<'de, T: Deserialize<'de>, B: Buf>(
    buf: &'de mut B,
) -> Result<T, Error> {
    let mut deserializer = de::Deserializer::new(buf);
    T::deserialize(&mut deserializer)
}

pub fn serialized_size<T: Serialize>(val: &T) -> Result<usize, Error> {
    let mut computor = size::ComputeSize::new();
    val.serialize(&mut computor)?;
    Ok(computor.size())
}

pub fn write_to<T: Serialize>(val: &T, buf: &mut BytesMut) -> Result<(), Error> {
    let size = serialized_size(val)?;
    buf.reserve(size);
    write_to_no_resize(val, buf)
}

pub fn write_to_no_resize<T: Serialize, B: BufMut>(
    val: &T,
    buf: &mut B,
) -> Result<(), Error> {
    let mut serializer = ser::Serializer::new(buf);
    val.serialize(&mut serializer)?;
    Ok(())
}
