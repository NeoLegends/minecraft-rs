use crate::error::Error;
use bytes::Buf;
use minecraft_varint::VarReadExt;
use serde::de::{self, *};
use std::io::ErrorKind;

pub struct Deserializer<'de, B>(&'de mut B);

struct Seq<'a, 'de, B> {
    de: &'a mut Deserializer<'de, B>,
    len: usize,
}

impl<'de, B> Deserializer<'de, B> {
    pub fn new(buf: &'de mut B) -> Self {
        Deserializer(buf)
    }
}

impl<'de, B: Buf> Deserializer<'de, B> {
    fn read_boolean(&mut self) -> Result<bool, Error> {
        self.require_length(1)?;

        match self.0.get_u8() {
            0 => Ok(false),
            1 => Ok(true),
            v => Err(de::Error::invalid_value(
                Unexpected::Unsigned(v as u64),
                &"0 or 1",
            )),
        }
    }

    fn read_i32(&mut self) -> Result<i32, Error> {
        self.0.read_var_i32().map_err(|e| match e.kind() {
            ErrorKind::UnexpectedEof => Error::UnexpectedEndOfBuffer,
            e => panic!("unknown error during i32 varint read from memory: {:?}", e),
        })
    }

    fn read_i32_len(&mut self) -> Result<usize, Error> {
        let len = self.read_i32()?;

        if len < 0 {
            return Err(de::Error::invalid_value(
                Unexpected::Signed(len as i64),
                &"positive length",
            ));
        }

        Ok(len as usize)
    }

    fn read_bytes(&mut self) -> Result<Vec<u8>, Error> {
        let len = self.read_i32_len()?;
        self.require_length(len)?;

        let mut buf = vec![0; len];
        self.0.copy_to_slice(&mut buf);

        Ok(buf)
    }

    fn require_length(&self, len: usize) -> Result<(), Error> {
        if self.0.remaining() >= len {
            Ok(())
        } else {
            Err(Error::UnexpectedEndOfBuffer)
        }
    }
}

impl<'a, 'de, B: Buf> de::Deserializer<'de> for &'a mut Deserializer<'de, B> {
    type Error = Error;

    fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.read_boolean()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.require_length(1)?;
        visitor.visit_i8(self.0.get_i8())
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.require_length(1)?;
        visitor.visit_i16(self.0.get_i16_be())
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.read_i32()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.0.read_var_i64().map_err(|e| match e.kind() {
            ErrorKind::UnexpectedEof => Error::UnexpectedEndOfBuffer,
            _ => unreachable!(),
        })?;

        visitor.visit_i64(val)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.require_length(16)?;
        visitor.visit_i128(self.0.get_i128_be())
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.require_length(1)?;
        visitor.visit_u8(self.0.get_u8())
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.require_length(2)?;
        visitor.visit_u16(self.0.get_u16_be())
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.require_length(4)?;
        visitor.visit_u32(self.0.get_u32_be())
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.require_length(8)?;
        visitor.visit_u64(self.0.get_u64_be())
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.require_length(16)?;
        visitor.visit_u128(self.0.get_u128_be())
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.require_length(4)?;
        visitor.visit_f32(self.0.get_f32_be())
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.require_length(8)?;
        visitor.visit_f64(self.0.get_f64_be())
    }

    fn deserialize_char<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let bytes = self.read_bytes()?;
        let string = String::from_utf8(bytes).map_err(|e| {
            de::Error::invalid_value(
                Unexpected::Bytes(e.as_bytes()),
                &"valid UTF-8 bytes",
            )
        })?;

        visitor.visit_string(string)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.read_bytes()?)
    }

    fn deserialize_option<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let len = self.read_i32_len()?;
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Seq { de: self, len: len })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, 'de, B: Buf> EnumAccess<'de> for &'a mut Deserializer<'de, B> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let variant = i32::deserialize(&mut *self)?;

        if variant < 0 {
            return Err(de::Error::invalid_value(
                Unexpected::Signed(variant as i64),
                &"value >= 0",
            ));
        }

        let val = seed.deserialize((variant as u32).into_deserializer())?;
        Ok((val, self))
    }
}

impl<'a, 'de, B: Buf> VariantAccess<'de> for &'a mut Deserializer<'de, B> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        Err(Error::Unsupported)
    }

    fn tuple_variant<V>(self, _: usize, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn struct_variant<V>(
        self,
        _: &'static [&'static str],
        _: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }
}

impl<'a, 'de, B: Buf> SeqAccess<'de> for Seq<'a, 'de, B> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            return Ok(None);
        }

        self.len -= 1;
        let value = serde::de::DeserializeSeed::deserialize(seed, &mut *self.de)?;
        Ok(Some(value))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}
