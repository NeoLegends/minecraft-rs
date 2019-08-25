use crate::error::Error;
use bytes::BufMut;
use minecraft_varint::{
    var_i32_length, var_i64_length, var_usize_length, VarWriteExt,
};
use serde::ser::{self, *};
use std::{i32, io::ErrorKind};

#[derive(Debug)]
pub struct Serializer<'a, B>(&'a mut B);

impl<'a, B> Serializer<'a, B> {
    pub fn new(buf: &'a mut B) -> Self {
        Serializer(buf)
    }
}

impl<'a, B: BufMut> Serializer<'a, B> {
    fn require_capacity(&self, capacity: usize) -> Result<(), Error> {
        if self.0.remaining_mut() >= capacity {
            Ok(())
        } else {
            Err(Error::UnexpectedEndOfBuffer)
        }
    }
}

impl<'ser, 'a, B: BufMut> ser::Serializer for &'ser mut Serializer<'a, B> {
    type Ok = ();
    type Error = Error;

    type SerializeMap = Self;
    type SerializeSeq = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.serialize_u8(if v { 1 } else { 0 })
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(1)?;

        self.0.put(v);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(2)?;

        self.0.put_i16_be(v);
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(var_i32_length(v))?;

        self.0.write_var_i32(v).map_err(|e| match e.kind() {
            ErrorKind::UnexpectedEof => Error::UnexpectedEndOfBuffer,
            _ => unreachable!(),
        })
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(var_i64_length(v))?;

        self.0.write_var_i64(v).map_err(|e| match e.kind() {
            ErrorKind::UnexpectedEof => Error::UnexpectedEndOfBuffer,
            _ => unreachable!(),
        })
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(16)?;

        self.0.put_i128_be(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(1)?;

        self.0.put(v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(2)?;

        self.0.put_u16_be(v);
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(4)?;

        self.0.put_u32_be(v);
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(8)?;

        self.0.put_u64_be(v);
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(16)?;

        self.0.put_u128_be(v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(4)?;

        self.0.put_f32_be(v);
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(8)?;

        self.0.put_f64_be(v);
        Ok(())
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::Unsupported)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.require_capacity(var_usize_length(v.len()) + v.len())?;

        self.serialize_i32(v.len() as i32)?;
        self.0.put(v);

        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::Unsupported)
    }

    fn serialize_some<T: ?Sized>(self, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::Unsupported)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(
        self,
        _: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        // MC likes 1-based indices for enums
        (variant_index as i32 + 1).serialize(self)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::Unsupported)
    }

    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> Result<Self::SerializeSeq, Self::Error> {
        let len = match len {
            Some(l) => l,
            None => return Err(Error::LengthRequired),
        };

        if len > (i32::MAX as usize) {
            let msg =
                "list too long, max contain at max i32::MAX elements".to_owned();
            return Err(Error::Custom(msg));
        }

        self.serialize_i32(len as i32)?;
        Ok(self)
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::Unsupported)
    }

    fn serialize_map(
        self,
        _: Option<usize>,
    ) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::Unsupported)
    }

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::Unsupported)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'ser, 'a, B: BufMut> SerializeSeq for &'ser mut Serializer<'a, B> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'ser, 'a, B: BufMut> ser::SerializeTuple for &'ser mut Serializer<'a, B> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'ser, 'a, B: BufMut> ser::SerializeTupleStruct for &'ser mut Serializer<'a, B> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'ser, 'a, B: BufMut> ser::SerializeTupleVariant
    for &'ser mut Serializer<'a, B>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::Unsupported)
    }
}

impl<'ser, 'a, B: BufMut> ser::SerializeMap for &'ser mut Serializer<'a, B> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported)
    }

    fn serialize_value<T>(&mut self, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::Unsupported)
    }
}

impl<'ser, 'a, B: BufMut> ser::SerializeStruct for &'ser mut Serializer<'a, B> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'ser, 'a, B: BufMut> ser::SerializeStructVariant
    for &'ser mut Serializer<'a, B>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Unsupported)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::Unsupported)
    }
}
